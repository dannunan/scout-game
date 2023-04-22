#![allow(dead_code)]
#![allow(unused_variables)]

use scout::Config;
use std::env;
use std::process;

fn main() {
    println!("Scout!");

    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Error parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = scout::run(config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
