use std::process;
mod strategies;

fn main() {
    println!("Scout!");

    let mut strategies: Vec<Box<dyn scout_game::Strategy>> = vec![
        Box::new(strategies::GetPlayerAction::new()),
        Box::new(strategies::StrategyRush::new()),
        Box::new(strategies::StrategyRush::new()),
        Box::new(strategies::StrategyRush::new()),
    ];

    match scout_game::watch(&mut strategies, false) {
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
