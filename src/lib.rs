#![allow(dead_code)]
#![allow(unused_variables)]

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, VecDeque};
use std::{error::Error, io};

#[derive(Debug, Clone)]
pub struct Card(i32, i32);

impl Card {
    fn flip(&self) -> Card {
        Card(self.1, self.0)
    }
}

type Set = VecDeque<Card>;

/// Each player has a hand, some points, and their "Scout show" move.
#[derive(Debug, Default, Clone)]
pub struct Player {
    hand: Set,
    score: i32,
    scout_show: bool,
}

/// Player actions on a given turn.
pub enum Action {
    /// Scouting moves a card from the active set into the hand (it may be flipped)
    /// Specified with (left, flip, insert)
    Scout(bool, bool, usize),
    /// Showing replaces the active set with a stronger set from the hand
    /// Specified with (start, stop)
    Show(usize, usize),
    /// Scout and Show simply completes the other two actions in order
    /// Specified with (left, flip, insert, start, stop)
    ScoutShow(bool, bool, usize, usize, usize),
}

/// GameState for storing state and generating new state with Actions
#[derive(Debug, Default)]
pub struct GameState {
    players: VecDeque<Player>,
    game_size: usize,
    active: Set,
    active_owner: usize,
}

pub enum NewGameState {
    Continue(GameState),
    GameOver(Vec<i32>),
}

impl GameState {
    fn new(n: usize, shuffle: bool) -> Self {
        println!("Creating {} player game", n);

        let mut game = GameState {
            active: Set::new(),
            game_size: n,
            players: VecDeque::from(vec![Default::default(); n]),
            active_owner: 0,
        };

        let mut deck = create_deck(n, true);

        println!("Using {} card deck", deck.len());
        if shuffle {}

        // Deal out all cards
        let mut player_index = 0;
        for card in deck.drain(..) {
            game.players[player_index].hand.push_back(card);
            player_index = (player_index + 1) % n;
        }
        return game;
    }

    fn scout(&self, left: bool, flip: bool, index: usize) -> GameState {
        let mut players = self.players.clone();
        let game_size = self.game_size;
        let mut active = self.active.clone();
        let active_owner = self.active_owner;

        let mut card: Card;
        if left {
            card = active.pop_front().unwrap();
        } else {
            card = active.pop_back().unwrap();
        }
        if flip {
            card = card.flip();
        }
        players[0].hand.insert(index, card);
        players[active_owner].score += 1;

        GameState {
            players,
            game_size,
            active,
            active_owner,
        }
    }

    fn show(&self, start: usize, stop: usize) -> GameState {
        let mut players = self.players.clone();
        let game_size = self.game_size;
        let mut active = self.active.clone();
        let active_owner = 0;

        players[0].score += active.len() as i32;
        active.clear();
        for _ in start..stop + 1 {
            active.push_back(players[0].hand.remove(start).unwrap())
        }

        GameState {
            players,
            game_size,
            active,
            active_owner,
        }
    }

    fn take_action(&self, action: &Action) -> NewGameState {
        let mut state: GameState;
        match action {
            Action::Scout(left, flip, index) => state = self.scout(*left, *flip, *index),
            Action::Show(start, stop) => state = self.show(*start, *stop),
            Action::ScoutShow(left, flip, index, start, stop) => {
                state = self.scout(*left, *flip, *index).show(*start, *stop)
            }
        };

        // Check if game is over
        let hand_size = state.players[0].hand.len();
        // Round ends if hand is empty
        if hand_size == 0 {
            let mut scores = Vec::new();
            for i in 0..state.players.len() {
                scores.push(state.players[i].score - state.players[i].hand.len() as i32);
            }
            return NewGameState::GameOver(scores);
            // Round ends if active player is next player
        } else if state.active_owner == 1 {
            // The next player isn't penalised for hand size -
            // in this case, offset this players points by hand size
            state.players[1].score += hand_size as i32;
            let mut scores = Vec::new();
            for i in 0..state.players.len() {
                scores.push(state.players[i].score - state.players[i].hand.len() as i32);
            }
            return NewGameState::GameOver(scores);
        } else {
            return NewGameState::Continue(state);
        }
    }

    fn rotate_left(&mut self) {
        // Maybe combine with check victory to return some kind of Result?
        self.players.rotate_left(1);
        self.active_owner = (self.active_owner + self.game_size - 1) % self.game_size;
    }
}

fn print_set(set: &Set) {
    for card in set {
        print!("{}  |", card.0)
    }
    print!("\n");
    for card in set {
        print!("  {}|", card.1)
    }
    print!("\n");
}

/// Strategies are ways of generating Actions based on GameState
pub type Strategy = fn(&GameState) -> Action;

pub fn run(strategies: Vec<Strategy>) -> Result<Vec<i32>, Box<dyn Error>> {
    let n_players = strategies.len();
    let mut game = GameState::new(n_players, true);
    let mut turn = 0;

    loop {
        let action = get_player_action(&game);
        match game.take_action(&action) {
            NewGameState::Continue(new) => game = new,
            NewGameState::GameOver(mut scores) => {
                scores.rotate_left(turn);
                return Ok(scores);
            }
        };
        game.rotate_left();
        turn = (turn + 1) % n_players;
    }
}

fn create_deck(game_size: usize, shuffle: bool) -> Set {
    let mut deck: Set = VecDeque::new();
    match game_size {
        3 => {
            // Each unique combination of 0-8, excluding matches
            for bottom in 0..9 {
                for top in 0..bottom {
                    deck.push_back(Card(top, bottom));
                }
            }
        }
        4 => {
            // Each unique combination of 0-9, excluding matches and (9/8)
            for bottom in 0..10 {
                for top in 0..bottom {
                    deck.push_back(Card(top, bottom));
                }
            }
            deck.pop_back();
        }
        5 => {
            // Each unique combination of 0-9, excluding matches
            for bottom in 0..10 {
                for top in 0..bottom {
                    deck.push_back(Card(top, bottom));
                }
            }
        }
        _ => {}
    }
    if shuffle {
        deck.make_contiguous().shuffle(&mut thread_rng());
    }
    return deck;
}

pub fn generate_set_map() -> HashMap<Vec<i32>, i32> {
    // Generate all legal sets, and assign an i32 value to each.
    // This is done by generating the sets in order of their value

    let mut map = HashMap::new();
    let mut i = 0;

    // For size=1 straights and flushes are identical
    for base in 1..10 {
        i = i + 1;
        map.insert(vec![base], i);
    }

    // Iterate up to max Set size
    for size in 2..10 {
        // First add all straights (ascending and descending)
        for base in 1..10 {
            i = i + 1;
            map.insert((base..=size).collect(), i);
            map.insert((base..=size).rev().collect(), i);
        }
        // Then add all flushes (this is done after to preserve order)
        for base in 1..10 {
            i = i + 1;
            map.insert(vec![base; size.try_into().unwrap()], i);
        }
    }

    return map;
}

pub fn get_player_action(state: &GameState) -> Action {
    // Print some info
    println!("\nActive Set:");
    print_set(&state.active);
    println!("\nPoints: {} Hand:", state.players[0].score);
    print_set(&state.players[0].hand);
    let mut action = String::new();
    println!("\nSelect action:");
    io::stdin()
        .read_line(&mut action)
        .expect("Failed to read line");
    match action.trim() {
        "Scout" => Action::Scout(true, false, 0),
        "Show" => Action::Show(0, 0),
        "Scout and show" => Action::ScoutShow(true, false, 0, 0, 0),
        "Quit" => std::process::exit(0),
        _ => {
            println!("Not a valid action! Enter: Scout, Show, Scout and show, or Quit");
            get_player_action(state)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_init() {
        let game = GameState::new(3, false);
        assert_eq!(game.players[0].hand.len(), 12);
        let game = GameState::new(4, false);
        assert_eq!(game.players[0].hand.len(), 11);
        let game = GameState::new(5, false);
        assert_eq!(game.players[0].hand.len(), 9);
    }
}
