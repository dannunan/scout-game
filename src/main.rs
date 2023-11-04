use std::process;

fn main() {
    println!("Scout!");

    let strategies: Vec<scout::Strategy> = vec![
        scout::get_player_action,
        scout::strategy_random,
        scout::strategy_random,
        scout::strategy_random,
    ];

    match scout::run(&strategies) {
        Ok(scores) => {
            println!("Game over! Scores: {:?}", scores);
            process::exit(0);
        }
        Err(game) => {
            println!("Game halted!: {:?}", game);
            process::exit(1);
        }
    }
}
