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

    if let Err(e) = scout::run(strategies) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
