use std::process;

fn main() {
    println!("Scout!");

    let strategies: Vec<scout::Strategy> = vec![
        scout::strategy_show_random,
        scout::strategy_true_random,
        scout::strategy_true_random,
    ];

    println!("{:?}", scout::evaluate_strategies(&strategies, 1000));

    match scout::run(&strategies) {
        Ok(game_result) => {
            println!(
                "Game over! Turn: {} Scores: {:?}",
                game_result.turn, game_result.scores
            );
        }
        Err(game) => {
            println!("Game halted!: {:?}", game);
            process::exit(1);
        }
    }
    process::exit(0);
}
