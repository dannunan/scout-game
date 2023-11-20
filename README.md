# scout-game
Rust implementation of the card game **Scout**, originally designed by Kei Kanjino and published by Oink Games inc.

## Scout
This readme is not intended as a game manual, and will assume familiarity with the rules.

Scout is a trick taking card game for 3-5 players. In Scout players score points by creating and playing sets of cards. These sets may be either flushes (0,0,0) or straights (0,1,2).

A core mechanic in scout is that players hands **cannot be reordered**. To create strong sets, players must therefore add necessary cards to their hand (Scout), or remove obstructions (Show).

This module uses card values ranging from 0-9, instead of the original 1-10.

## Command Line Interface
Running from the command-line will start a game against 3 computer players. When prompted for an action, enter one of the following actions:
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
The primary reason for importing to a script would be to implement custom `Strategy` functions, to play against them or test them against the built-in computer players.

To create a game instance, pass a vector of strategy functions to `scout::run`. You can add a human player with the `get_player_action` strategy. The number of strategies determines the number of players, which must be between 3 and 5.

```rust
let strategies: Vec<scout::Strategy> = vec![
        scout::get_player_action,
        scout::strategy_rush,
        scout::strategy_rush,
        scout::strategy_rush,
    ];

    match scout::run(&strategies) {
        Ok(game_result) => {
            println!("Game over! Scores: {:?}", game_result.scores);
        }
        Err(game) => {
            println!("Game halted!: {:?}", game);
            process::exit(1);
        }
    }
```