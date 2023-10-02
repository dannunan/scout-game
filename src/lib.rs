#![allow(dead_code)]
#![allow(unused_variables)]

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::{error::Error, io};

enum GameSize {
    /// 3 players with 12 cards.
    THREE,
    /// 4 players with 11 cards.
    FOUR,
    /// 5 players with 9 cards.
    FIVE,
}

impl GameSize {
    fn as_usize(&self) -> usize {
        match self {
            Self::THREE => 3,
            Self::FOUR => 4,
            Self::FIVE => 5,
        }
    }
}

/// Player actions on a given turn.
enum Action {
    /// Scouting moves a card from the active set into the hand (it may be flipped)
    Scout(usize, bool, usize),
    /// Showing replaces the active set with a stronger set from the hand
    Show(usize, usize),
    /// Scout and Show simply completes the other two actions in order
    ScoutShow(usize, bool, usize, usize, usize),
    /// Exit
    Quit,
}

#[derive(Debug, Clone)]
pub struct Card(i32, i32);

fn flip(card: Card) -> Card {
    Card(card.1, card.0)
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

type Set = Vec<Card>;
type Hand = Vec<Card>;

pub fn generate_set_map() -> HashMap<Vec<i32>, i32> {
    // Generate all legal sets, and assign an i32 value to each.
    // This is done by generating the sets in order of their value

    // All single card sets
    // All runs of len 2

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

    map
}

/// Each player has a hand, some points, and their "Scout show" move.
#[derive(Debug, Default, Clone)]
pub struct Player {
    hand: Hand,
    score: i32,
    scoutshow: bool,
}

impl Player {
    fn draw(&mut self, card: Card) {
        // Drawing is not actually required during the game - should this be moved to new()?
        self.hand.push(card)
    }

    fn print_hand(&self) {
        print!("Hand: ");
        for card in &self.hand {
            print!(" {}", card)
        }
        println!("");
    }

    fn scout(&mut self, mut card: Card, index: i32, flip_card: bool) {
        if flip_card {
            card = flip(card)
        }
        self.hand.insert(index.try_into().unwrap(), card)
    }
}

/// A single instance of a Scout game.
#[derive(Debug, Default)]
pub struct GameState {
    deck: Vec<Card>,
    players: Vec<Player>,
    active: Vec<Card>,
    active_owner: usize,
    first_player: usize,
}

fn create_deck(game_size: &GameSize) -> Vec<Card> {
    let mut deck: Vec<Card> = Vec::new();
    match game_size {
        GameSize::THREE => {
            // Each unique combination of 1-9, excluding matches
            for bottom in 1..10 {
                for top in 1..bottom {
                    deck.push(Card(top, bottom));
                }
            }
        }
        GameSize::FOUR => {
            // Each unique combination of 1-10, excluding matches and (10/9)
            for bottom in 1..10 {
                for top in 1..bottom {
                    deck.push(Card(top, bottom));
                }
            }
            for top in 1..9 {
                deck.push(Card(top, 10));
            }
        }
        GameSize::FIVE => {
            // Each unique combination of 1-10, excluding matches
            for bottom in 1..11 {
                for top in 1..bottom {
                    deck.push(Card(top, bottom));
                }
            }
        }
    }
    // Do shuffle?
    deck
}

impl GameState {
    fn new(n: GameSize, shuffle: bool) -> Self {
        println!("Creating {} player game", n.as_usize());

        let mut game = GameState {
            active: Vec::<Card>::new(),
            deck: create_deck(&n),
            players: vec![Default::default(); n.as_usize()],
            active_owner: 0,
            first_player: 0,
        };

        println!("Using {} card deck", game.deck.len());
        if shuffle {
            game.deck.shuffle(&mut thread_rng());
        }
        game.deal();
        game
    }

    fn deal(&mut self) {
        let n_players = self.players.len();
        let mut player_index = 0;
        for card in self.deck.drain(..) {
            self.players[player_index].draw(card);
            player_index = (player_index + 1) % n_players;
        }
    }

    fn scout(&mut self, index: usize) -> Card {
        self.active.remove(index)
    }

    fn show(&mut self, set: Vec<Card>) -> usize {
        let score = self.active.len();
        self.active = set;
        score
    }
}

// Strategies are ways of generating Actions based on GameState

type Strategy = fn(GameState) -> Action;

fn get_player_action(state: GameState) -> Action {
    let mut action = String::new();
    println!("Select action:");
    io::stdin()
        .read_line(&mut action)
        .expect("Failed to read line");
    match action.trim() {
        "Scout" => Action::Scout(0, false, 0),
        "Show" => Action::Show(0, 0),
        "Scout and show" => Action::ScoutShow(0, false, 0, 0, 0),
        "Quit" => Action::Quit,
        _ => {
            println!("Not a valid action!");
            println!("{}", action.as_str());
            get_player_action(state)
        }
    }
}

/// Config struct to sanitize user input.
pub struct Config {
    gamesize: GameSize,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &str> {
        let gamesize = match args[1].trim() {
            "3" => GameSize::THREE,
            "4" => GameSize::FOUR,
            "5" => GameSize::FIVE,
            _ => return Err("Invalid game size"),
        };
        Ok(Config { gamesize })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let game = GameState::new(config.gamesize, true);

    game.players[0].print_hand();
    let action = get_player_action(game);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_init() {
        let game = GameState::new(GameSize::THREE, false);
        assert_eq!(game.players[0].hand.len(), 12);
        let game = GameState::new(GameSize::FOUR, false);
        assert_eq!(game.players[0].hand.len(), 11);
        let game = GameState::new(GameSize::FIVE, false);
        assert_eq!(game.players[0].hand.len(), 9);
    }
}
