#[cfg(test)]
pub mod tests;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use log::{info, warn};
use rand::prelude::SliceRandom;
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct AppData {
    #[arg(short, long)]
    secret: String,

    #[arg(skip)]
    version: String,

    #[arg(short, long)]
    port: u16,

    #[arg(short, long, default_value = "info")]
    log: String,

    #[arg(short, long, default_value = "false")]
    debug: bool,
}

trait StringMinify {
    fn minify(&self) -> String;
}

impl StringMinify for String {
    fn minify(&self) -> String {
        self.replace("\n", "")
            .replace("    ", "")
            .replace(": ", ":")
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Rust Tic Tac Toe Bot!")
}

#[derive(Deserialize)]
pub struct QueryData {
    method: String,
}

#[post("/")]
async fn handle_request(
    body: String,
    config: web::Data<AppData>,
    query: web::Query<QueryData>,
) -> impl Responder {
    let response = match query.method.as_str() {
        "status" => handle_status(body, &config),
        "finish" => handle_finish(body, &config),
        "error" => handle_error(body, &config),
        "start" => handle_start(body, &config),
        "turn" => handle_turn(body, &config),
        _ => {
            unimplemented!("Unimplemented method: {}", query.method)
        }
    };

    return HttpResponse::Ok().body(response);
}

pub fn handle_status(_body: String, config: &AppData) -> String {
    info!("Handling status request");

    return format!(
        r#"{{
        "status": "ok",
        "game": "tic-tac-toe",
        "version": "{}",
        "secret": "{}",
        "message": "I'm ready!"
    }}"#,
        config.version, config.secret
    )
    .minify();
}

pub fn handle_finish(_body: String, _config: &AppData) -> String {
    info!("Handling finish request");

    return r#"{
        "status": "ok",
        "message": "Game finished!"
    }"#
    .to_string()
    .minify();
}

pub fn handle_error(body: String, _config: &AppData) -> String {
    info!("Handling error request: {}", body);

    return r#"{
        "status": "error",
        "message": "Invalid request!"
    }"#
    .to_string()
    .minify();
}

pub fn handle_start(_body: String, config: &AppData) -> String {
    info!("Handling start request");

    return format!(
        r#"{{
        "status": "ok",
        "game": "tic-tac-toe",
        "version": "{}",
        "secret": "{}",
        "accept": true,
        "message": "Let's go!"
    }}"#,
        config.version, config.secret
    )
    .minify();
}

#[derive(Deserialize)]
pub struct TurnData {
    turn_number: u8,
    figure: char,
    board: [[i16; 3]; 3],
}

pub fn handle_turn(body: String, config: &AppData) -> String {
    info!("Handling turn request");

    let mut file: Option<std::fs::File> = None;
    if config.debug {
        file = Some(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open("log.dot")
                .expect("Failed to open log file"),
        );
    }

    let data: TurnData = serde_json::from_str(&body).unwrap();
    let figure: i16 = if data.figure == 'X' { 1 } else { 2 };
    let allowed_moves: Vec<[usize; 2]> = get_allowed_moves(data.board);
    let mut best_score = if figure == 1 {
        std::i16::MIN
    } else {
        std::i16::MAX
    };
    let mut best_move: Option<[usize; 2]> = None;
    let original_board = data.board.clone();

    if let Some(ref mut file) = file {
        writeln!(file, "digraph G {{").unwrap();
    }

    if data.turn_number == 0 {
        best_move = Some([0, 0]);
    } else {
        for move_ in &allowed_moves {
            let mut board = original_board.clone();
            board[move_[0]][move_[1]] = figure;
            let score = minimax(&mut file, board, 0, figure == 2, figure);

            if let Some(ref mut file) = file {
                write_board(file, data.board, board, score);
            }

            if figure == 1 {
                if score > best_score {
                    best_score = score;
                    best_move = Some(move_.clone());
                }
            } else {
                if score < best_score {
                    best_score = score;
                    best_move = Some(move_.clone());
                }
            }
        }
    }

    if best_move.is_none() {
        warn!("No best move found, choosing random move");
        best_move = Some(
            allowed_moves
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone(),
        );
    }

    if let Some(ref mut file) = file {
        writeln!(
            file,
            "\"{}\" -> \"{}\" [lable=\"{}\" color=\"red\"]",
            board_to_string(data.board),
            board_to_string_move(data.board, best_move.unwrap(), figure),
            best_score
        )
        .unwrap();

        writeln!(file, "}}").unwrap();
    }

    return format!(
        r#"{{
        "status": "ok",
        "game": "tic-tac-toe",
        "version": "{}",
        "secret": "{}",
        "move": [{}, {}]
    }}"#,
        config.version,
        config.secret,
        best_move.unwrap()[0],
        best_move.unwrap()[1]
    )
    .minify();
}

pub fn write_board(
    f: &mut std::fs::File,
    board: [[i16; 3]; 3],
    new_board: [[i16; 3]; 3],
    score: i16,
) {
    writeln!(
        f,
        "\"{}\" -> \"{}\" [label=\"{}\"]",
        board_to_string(board),
        board_to_string(new_board),
        score
    )
    .unwrap();
}

pub fn board_to_string_move(board: [[i16; 3]; 3], move_: [usize; 2], figure: i16) -> String {
    let mut new_board = board.clone();
    new_board[move_[0]][move_[1]] = figure;

    return board_to_string(new_board);
}

pub fn board_to_string(board: [[i16; 3]; 3]) -> String {
    let mut result = String::new();
    for row in board.iter() {
        for cell in row.iter() {
            if *cell == 0 {
                result.push('_');
            } else if *cell == 1 {
                result.push('X');
            } else if *cell == 2 {
                result.push('O');
            }
        }
        result.push('\n');
    }

    return result;
}

pub fn minimax(
    file: &mut Option<std::fs::File>,
    given_board: [[i16; 3]; 3],
    depth: i32,
    is_maximizing: bool,
    figure: i16,
) -> i16 {
    let winner = check_winner(given_board);
    if winner != 0 {
        return winner * 100;
    }

    if given_board
        .iter()
        .all(|row| row.iter().all(|cell| *cell != 0))
    {
        return 0;
    }

    let reverse_figure: i16 = if figure == 1 { 2 } else { 1 };

    if is_maximizing {
        let mut best_score = std::i16::MIN;
        for i in 0..3 {
            for j in 0..3 {
                if given_board[i][j] == 0 {
                    let mut new_board = given_board.clone();
                    new_board[i][j] = reverse_figure;
                    let num_of_pieces: usize = new_board
                        .iter()
                        .flatten()
                        .filter(|cell| **cell != 0)
                        .count();
                    let score: i16 = minimax(file, new_board, depth + 1, false, reverse_figure)
                        - num_of_pieces as i16;
                    if score > best_score {
                        best_score = score;
                    }

                    if depth < 1 {
                        if let Some(ref mut file) = file {
                            write_board(file, given_board, new_board, score);
                        }
                    }
                }
            }
        }
        return best_score;
    } else {
        let mut best_score = std::i16::MAX;
        for i in 0..3 {
            for j in 0..3 {
                if given_board[i][j] == 0 {
                    let mut new_board = given_board.clone();
                    new_board[i][j] = reverse_figure;
                    let num_of_pieces: usize = new_board
                        .iter()
                        .flatten()
                        .filter(|cell| **cell != 0)
                        .count();
                    let score: i16 = minimax(file, new_board, depth + 1, true, reverse_figure)
                        + num_of_pieces as i16;
                    if score < best_score {
                        best_score = score;
                    }

                    if depth < 1 {
                        if let Some(ref mut file) = file {
                            write_board(file, given_board, new_board, score);
                        }
                    }
                }
            }
        }
        return best_score;
    }
}

pub fn check_winner(board: [[i16; 3]; 3]) -> i16 {
    let winning_combinations = [
        [(0, 0), (0, 1), (0, 2)],
        [(1, 0), (1, 1), (1, 2)],
        [(2, 0), (2, 1), (2, 2)],
        [(0, 0), (1, 0), (2, 0)],
        [(0, 1), (1, 1), (2, 1)],
        [(0, 2), (1, 2), (2, 2)],
        [(0, 0), (1, 1), (2, 2)],
        [(0, 2), (1, 1), (2, 0)],
    ];

    for combination in winning_combinations {
        let mut x_count = 0;
        let mut o_count = 0;
        for cell in combination.iter() {
            let (i, j) = cell;
            if board[*i][*j] == 1 {
                x_count += 1;
            } else if board[*i][*j] == 2 {
                o_count += 1;
            }
        }

        if x_count == 3 {
            return 1;
        } else if o_count == 3 {
            return -1;
        }
    }

    return 0;
}

pub fn get_allowed_moves(board: [[i16; 3]; 3]) -> Vec<[usize; 2]> {
    let mut allowed_moves: Vec<[usize; 2]> = Vec::new();

    for i in 0..3 {
        for j in 0..3 {
            if board[i][j] == 0 {
                allowed_moves.push([i, j]);
            }
        }
    }

    return allowed_moves;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut app_data = AppData::parse();
    app_data.version = env!("CARGO_PKG_VERSION").to_string();
    let port = app_data.clone().port;
    std::env::set_var("RUST_APP_LOG", app_data.clone().log);
    pretty_env_logger::init_custom_env("RUST_APP_LOG");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .service(hello)
            .service(handle_request)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
