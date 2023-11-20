use std::process;

fn main() {
    println!("Scout!");

    let strategies: Vec<scout_game::Strategy> = vec![
        scout_game::get_player_action,
        scout_game::strategy_rush,
        scout_game::strategy_rush,
        scout_game::strategy_rush,
    ];

    match scout_game::run(&strategies) {
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
