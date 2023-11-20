use std::process;

fn main() {
    println!("Scout!");

    let strategies: Vec<scout::Strategy> = vec![
        scout::get_player_action,
        scout::strategy_rush,
        scout::strategy_rush,
        scout::strategy_rush,
    ];

    // println!("{:?}", scout::evaluate_strategies(&strategies, 1000));

    match scout::watch(&strategies) {
        Ok(game_result) => {
            println!("Game over! Scores: {:?}", game_result.scores);
        }
        Err(game) => {
            println!("Game halted!: {:?}", game);
            process::exit(1);
        }
    }
    process::exit(0);
}
