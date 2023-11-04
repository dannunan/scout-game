#![allow(dead_code)]
#![allow(unused_variables)]

use std::process;

fn main() {
    println!("Scout!");

    let strategies: Vec<scout::Strategy> = vec![
        scout::get_player_action,
        scout::get_player_action,
        scout::get_player_action,
        scout::get_player_action,
    ];

    match scout::run(strategies) {
        Ok(scores) => {
            println!("Game over! Scores: {:?}", scores);
            process::exit(0);
        }
        Err(e) => {
            println!("Application error: {}", e);
            process::exit(1);
        }
    }
}
