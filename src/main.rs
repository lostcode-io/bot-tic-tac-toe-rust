use dotenv::dotenv;
use std::env;
use threadpool::ThreadPool;
use std::{
    net::{TcpListener, TcpStream},
    io::prelude::*,
};

#[derive(Clone)]
pub struct Config {
    version: String,
    port: u16,
    secret: String,
    threads: usize,
}
impl Config {
    pub fn new() -> Self {
	dotenv().ok();
	let port: u16 = env::var("PORT").expect("PORT must be set in .env")
	    .parse().expect("PORT must be a number");
	// Check if port < 65535 is not needed, because u16 is always less than 65535
	assert!(port > 1024, "PORT must be between 1024 and 65535");
	let threads: usize = env::var("THREADS").expect("THREADS must be set in .env")
	    .parse().expect("THREADS must be a number");
	assert!(threads > 0, "THREADS must be greater than 0");
	Config {
	    version: env::var("VERSION").expect("VERSION must be set in .env"),
	    port,
	    secret: env::var("SECRET").expect("SECRET must be set in .env"),
	    threads,
	}
    }

    pub fn get_version(&self) -> &str {
	&self.version
    }

    pub fn get_port(&self) -> u16 {
	self.port
    }

    pub fn get_port_str(&self) -> String {
	self.port.to_string()
    }

    pub fn get_secret(&self) -> &str {
	&self.secret
    }
}

fn main() {
    let config = Config::new();
    let listener = TcpListener::bind(
	"localhost:".to_string() + config.get_port_str().as_str()
    ).expect("Failed to create listener (Is the port free?)");
    println!("Listening on port {}", config.get_port());
    let pool = ThreadPool::new(config.threads);

    for stream in listener.incoming() {
	let cloned_config = config.clone();
	pool.execute(move || {
            connection_handler(stream.unwrap(), cloned_config);
	});
    }
}

fn connection_handler(mut stream: TcpStream, config: Config) {
    let mut buffer: [u8; 2048] = [0; 2048];
    stream.read(&mut buffer).unwrap();
    let lines_unformatted: String = String::from_utf8_lossy(&buffer).to_string();
    let mut lines: String = String::new();
    for c in lines_unformatted.chars() {
	if c == '\0' {
	    break;
	}
	lines.push(c);
    }
    let lines: Vec<&str> = lines.split("\r\n").collect();
    println!("Received request: {:?}", lines);

    let mut method: String = String::new();
    let mut path: String = String::new();
    let mut content_type: String = String::new();
    let mut content_length: usize = 0;
    let mut read_body: bool = false;
    let mut body_vec: Vec<String> = Vec::new();
    let mut i: i32 = -1;
    for l in lines {
	i += 1;
	let line: String = l.to_string();
	if i == 0 {
	    method = line.split(" ").nth(0).unwrap().to_string();
	    path = line.split(" ").nth(1).unwrap().to_string();
	    continue;
	}
	if line.is_empty() && !read_body {
	    read_body = true;
	    continue;
	}
	if read_body {
	    body_vec.push(line);
	} else {
	    let header_name: String = line.split(":").nth(0).unwrap().trim().to_string();
	    let header_value: String = line.split(":").nth(1).unwrap().trim().to_string();
	    match header_name.as_str() {
		"Content-Type" => {
		    content_type = header_value;
		},
		"Content-Length" => {
		    content_length = header_value.parse::<usize>().unwrap();
		},
		_ => {
		    continue;
		}
	    }
	}
    }
    let body: String = body_vec.join("\n");

    println!("Method: `{}`", method);
    println!("Path: `{}`", path);
    println!("Content-Type: `{}`", content_type);
    println!("Content-Length: `{}`", content_length);
    println!("Body: `{}`", body);

    if method.is_empty() || method != "POST" {
	stream.write("HTTP/1.1 400 Bad Request\r\n\r\n".as_bytes()).unwrap();
	return;
    }
    if path.is_empty() {
	stream.write("HTTP/1.1 400 Bad Request\r\n\r\n".as_bytes()).unwrap();
	return;
    }
    if content_type.is_empty() || !content_type.contains("application/json") {
	stream.write("HTTP/1.1 400 Bad Request\r\n\r\n".as_bytes()).unwrap();
	return;
    }

    match path.as_str() {
	"/?method=status" => {
	    stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n".as_bytes()).unwrap();
	    stream.write(format!("{{
                \"status\": \"ok\",
                \"game\": \"tic-tac-toe\",
                \"version\": \"{}\",
                \"secret\": \"{}\",
                \"message\": \"Iron is rusted, ready to go!\"
            }}", config.get_version(), config.get_secret()).as_bytes()).unwrap();
	    println!("Handled status request");
	},
	"/?method=start" => {
	    stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n".as_bytes()).unwrap();
	    stream.write(format!("{{
                \"status\": \"ok\",
                \"game\": \"tic-tac-toe\",
                \"version\": \"{}\",
                \"secret\": \"{}\",
                \"accept\": true,
                \"message\": \"Iron is rusted, ready to go!\"
            }}", config.get_version(), config.get_secret()).as_bytes()).unwrap();
	    println!("Handled start request");
	},
	"/?method=finish" => {
	    println!("Handled finish request");
	},
	"/?method=turn" => {
	    let json: serde_json::Value = serde_json::from_str(body.as_str()).expect("JSON was not well-formatted");
	    let response = handle_turn(json, config);
	    stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n".as_bytes()).unwrap();
	    stream.write(response.as_bytes()).unwrap();
	    println!("Handled turn request");
	},
	_ => {
	    stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap();
	    println!("Unable to handle unknown request: \"{}\"", path);
	},
    }
}

fn handle_turn(data: serde_json::Value, config: Config) -> String {
    let board: [[usize; 3]; 3] = serde_json::from_value(data["board"].clone()).expect("Board was not well-formatted");
    println!("Received board: {:?}", board);
    let mut response: [usize; 2] = [0, 0];
    let mut iterations: usize = 0;
    loop {
	response[0] = rand::random::<usize>() % 3;
	response[1] = rand::random::<usize>() % 3;
	if board[response[0]][response[1]] == 0 {
	    break;
	}
	iterations += 1;
	if iterations > 100 {
	    break;
	}
    }
    return format!("{{
        \"status\": \"ok\",
        \"game\": \"tic-tac-toe\",
        \"version\": \"{}\",
        \"secret\": \"{}\",
        \"move\": [{}, {}]
    }}", config.get_version(), config.get_secret(), response[0], response[1]);
}
