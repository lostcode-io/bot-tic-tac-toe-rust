use crate::*;

#[test]
fn test_handle_status() {
        let config = AppData {
        secret: "secret".into(),
        port: 8080,
        version: "0.1.0".into(),
        debug: false,
        log: "error".into(),
    };

    // Handle status doesn't care about the request body
    let response = handle_status("".into(), &config);

    assert_eq!(
        response,
        format!(
            r#"{{"status":"ok","game":"tic-tac-toe","version":"{}","secret":"{}","message":"I'm ready!"}}"#,
            config.version, config.secret
        )
    );
}

#[test]
fn test_handle_finish() {
    let config = AppData {
        secret: "secret".into(),
        port: 8080,
        version: "0.1.0".into(),
        debug: false,
        log: "error".into(),
    };

    // Handle finish doesn't care about the request body
    let response = handle_finish("".into(), &config);

    assert_eq!(
        response,
        r#"{"status":"ok","message":"Game finished!"}"#,
    );
}

#[test]
fn test_handle_start() {
    let config = AppData {
        secret: "secret".into(),
        port: 8080,
        version: "0.1.0".into(),
        debug: false,
        log: "error".into(),
    };

    // Handle start doesn't care about the request body
    let response = handle_start("".into(), &config);

    assert_eq!(
        response,
        format!(
            r#"{{"status":"ok","game":"tic-tac-toe","version":"{}","secret":"{}","accept":true,"message":"Let's go!"}}"#,
            config.version, config.secret
        )
    );
}

#[test]
fn test_handle_error() {
    let config = AppData {
        secret: "secret".into(),
        port: 8080,
        version: "0.1.0".into(),
        debug: false,
        log: "error".into(),
    };
    let body = r#"{"status":"error"}"#;

    let response = handle_error(body.into(), &config);

    assert_eq!(
        response,
        r#"{"status":"error","message":"Invalid request!"}"#,
    );
}

#[test]
fn test_handle_turn_first_move() {
    let config = AppData {
        secret: "secret".into(),
        port: 8080,
        version: "0.1.0".into(),
        debug: false,
        log: "error".into(),
    };
    let body = r#"{"game_id":723,"turn_number":0,"figure":"X","board":[[0,0,0],[0,0,0],[0,0,0]],"last_turns":[]}"#;

    let response = handle_turn(body.into(), &config);

    assert_eq!(
        response,
        format!(
            r#"{{"status":"ok","game":"tic-tac-toe","version":"{}","secret":"{}","move":[0, 0]}}"#,
            config.version, config.secret
        )
    );
}

#[test]
fn test_handle_turn_second_move() {
    let config = AppData {
        secret: "secret".into(),
        port: 8080,
        version: "0.1.0".into(),
        debug: false,
        log: "error".into(),
    };
    let body = r#"{"game_id":723,"turn_number":1,"figure":"O","board":[[1,0,0],[0,0,0],[0,0,0]],"last_turns":[{"turn_number":1,"player_id":1,"ego":true,"figure":"X","move":[0,0]}]}"#;

    let response = handle_turn(body.into(), &config);

    assert_eq!(
        response,
        format!(
            r#"{{"status":"ok","game":"tic-tac-toe","version":"{}","secret":"{}","move":[1, 1]}}"#,
            config.version, config.secret
        )
    );
}
