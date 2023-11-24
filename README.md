# scout-game
Rust implementation of the card game **Scout**, originally designed by Kei Kanjino and published by Oink Games inc.

## Scout
This readme is not intended as a game manual, and will assume familiarity with the rules.

Scout is a trick taking card game for 3-5 players. In Scout players score points by creating and playing sets of cards. These sets may be either flushes (0,0,0) or straights (0,1,2).

A core mechanic in scout is that players hands **cannot be reordered**. To create strong sets, players must therefore add necessary cards to their hand (Scout), or remove obstructions (Show).

This module uses card values ranging from 0-9, instead of the original 1-10.

## Command Line Interface
Running the package will start a game against 3 computer players. When prompted for an action, enter one of the following actions:
- `scout [left] [flip] [index]`
- `show [start] [stop]`
- `scoutshow [left] [flip] [index]`
- `quit`

All arguments should be numeric (1 representing `true`).

The **scout** action has arguments: `left` for which side of the active set to scout, `flip` if the card is to be flipped, and the `index` to insert the card at.

The **show** action has arguments `start` and `stop`, which are the inclusive bounds of the set to show. A single card can be played by repeating e.g. `show 2 2`.

The final action, **scoutshow**, is simply the above actions combined. You should first enter arguments for the scout step, then you will be presented with a new view and can input a show action.

Entering **quit** will cause the game to halt. This will print a debug view of the `GameState` before exiting.

## Library
To create a game instance, pass a vector of boxed strategy structs to `scout_game::run` or `scout_game::watch`. Both will run a single game, however `watch` prints information during the game.

Custom computer players can be created with structs which implement `Strategy`.
The current strategies are `GetPlayerAction` and `StrategyRush`.
`GetPlayerAction` prompts the user for actions, `StrategyRush` is a crude strategy which attempts to end the game as fast as possible.

The number of strategies determines the number of players, which must be between 3 and 5.

```rust

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
```