use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, VecDeque};
use std::{fmt, io};

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
#[derive(PartialEq, Clone, Copy)]
pub enum Action {
    /// Scouting moves a card from the active set into the hand (it may be flipped)
    /// Specified with (left, flip, insert)
    Scout(bool, bool, usize),
    /// Showing replaces the active set with a stronger set from the hand
    /// Specified with (start, stop) inclusive
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

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Scout(left, flip, insert) => {
                write!(f, "Scout l:{}, f:{}, i:{}", left, flip, insert)
            }
            Self::Show(start, stop) => {
                write!(f, "Show {} to {}", start, stop)
            }
            Self::ScoutShow(_, _, _, _, _) => {
                write!(f, "Scout and show!")
            }
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Score: {}, Hand: {:?}", self.score, top_only(&self.hand))
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut hands: String = "Hands\n".to_owned();
        for player in &self.players {
            hands.push_str(&format!("{}\n", player))
        }
        write!(f, "{}", hands)
    }
}

impl GameState {
    fn new(n: usize, shuffle: bool) -> Self {
        let mut game = GameState {
            active: Set::new(),
            game_size: n,
            players: VecDeque::from(vec![Default::default(); n]),
            active_owner: 0,
        };

        let mut deck = create_deck(n, shuffle);

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
                state = self.scout(*left, *flip, *index).show(*start, *stop);
                state.players[0].scout_show = false;
            }
        };

        // Round ends if current players hand is empty
        if state.players[0].hand.len() == 0 {
            let mut scores = Vec::new();
            for i in 0..state.players.len() {
                scores.push(state.players[i].score - state.players[i].hand.len() as i32);
            }
            return NewGameState::GameOver(scores);
            // Round ends if active owner is next player (1)
        } else if state.active_owner == 1 {
            // The next player isn't penalised for their hand size -
            // offset this players points by their hand size then count normally
            state.players[1].score += state.players[1].hand.len() as i32;
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
pub type Strategy = fn(&GameState, &SetMap) -> Option<Action>;

pub struct GameResult {
    pub scores: Vec<i32>,
    pub turn: usize,
}

pub fn run(strategies: &Vec<Strategy>) -> Result<GameResult, GameState> {
    let set_map = generate_set_map();
    let n_players = strategies.len();
    let mut game = GameState::new(n_players, true);
    let mut turn = 0;

    loop {
        let action = strategies[turn % n_players](&game, &set_map);
        match action {
            Some(action) => {
                match game.take_action(&action) {
                    NewGameState::Continue(new) => game = new,
                    NewGameState::GameOver(mut scores) => {
                        scores.rotate_right(turn % n_players);
                        return Ok(GameResult { scores, turn });
                    }
                };
            }
            None => {
                return Err(game);
            }
        }

        game.rotate_left();
        turn += 1;
    }
}

pub fn watch(strategies: &Vec<Strategy>) -> Result<GameResult, GameState> {
    let set_map = generate_set_map();
    let n_players = strategies.len();
    let mut game = GameState::new(n_players, true);
    let mut turn = 0;

    loop {
        if turn % n_players == 0 {
            println!("\nTurn {}", turn);
            println!("{}", game);
        }
        let action = strategies[turn % n_players](&game, &set_map);
        match action {
            Some(action) => {
                println!("Player {}: {}", turn % n_players, action);
                match game.take_action(&action) {
                    NewGameState::Continue(new) => game = new,
                    NewGameState::GameOver(mut scores) => {
                        scores.rotate_right(turn % n_players);
                        return Ok(GameResult { scores, turn });
                    }
                };
            }
            None => {
                return Err(game);
            }
        }

        game.rotate_left();
        turn += 1;
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

type SetMap = HashMap<Vec<i32>, i32>;

fn generate_set_map() -> SetMap {
    // Generate all legal sets, and assign an i32 value to each.
    // This is done by generating the sets in order of their value

    let mut map = HashMap::new();
    let mut i = 1;

    // For size=1 straights and flushes are identical
    for base in 0..10 {
        map.insert(vec![base], i);
        i = i + 1;
    }

    // Iterate up to max Set size
    for size in 2..10 {
        // First add all straights (ascending and descending)
        for base in 0..10 {
            map.insert((base..base + size).collect(), i);
            map.insert((base..base + size).rev().collect(), i);
            i = i + 1;
        }
        // Then add all flushes (this is done after to preserve order)
        for base in 0..10 {
            map.insert(vec![base; size as usize], i);
            i = i + 1;
        }
    }

    return map;
}

fn top_only(set: &Set) -> Vec<i32> {
    let mut vec = Vec::new();
    for card in set {
        vec.push(card.0)
    }
    return vec;
}

/// Get all valid Actions for player 0
fn get_valid_actions(state: &GameState, set_map: &SetMap) -> Vec<Action> {
    let mut actions = Vec::new();
    let player = &state.players[0];

    // Scout actions
    if !state.active.is_empty() {
        for i in 0..player.hand.len() {
            actions.push(Action::Scout(false, false, i));
            actions.push(Action::Scout(false, true, i));
            actions.push(Action::Scout(true, false, i));
            actions.push(Action::Scout(true, true, i));
        }
    }

    // Show actions
    let active_set_score = set_map.get(&top_only(&state.active)).unwrap_or(&0);
    let hand = top_only(&player.hand);
    for start in 0..hand.len() {
        for stop in start..hand.len() {
            if let Some(score) = set_map.get(&hand[start..stop + 1]) {
                if *score > *active_set_score {
                    actions.push(Action::Show(start, stop))
                }
            }
        }
    }

    return actions;
}

pub fn get_player_action(state: &GameState, set_map: &SetMap) -> Option<Action> {
    // Print some info
    println!("\nActive Set:");
    print_set(&state.active);
    println!("\nPoints: {} Hand:", state.players[0].score);
    print_set(&state.players[0].hand);
    for i in 0..state.players[0].hand.len() {
        print!(" {}  ", i)
    }
    print!("\n");
    let mut input = String::new();
    println!("\nSelect action:");
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let split: Vec<&str> = input.trim().split(" ").collect();
    let action = match split[0] {
        "Scout" => Action::Scout(split[1] == "1", split[2] == "1", split[3].parse().unwrap()),
        "Show" => Action::Show(split[1].parse().unwrap(), split[2].parse().unwrap()),
        "Scout and show" => Action::ScoutShow(true, false, 0, 0, 0),
        "Quit" => return None,
        _ => {
            println!("Input not accepted! Enter: Scout, Show, Scout and show, or Quit");
            return get_player_action(&state, set_map);
        }
    };
    if get_valid_actions(&state, set_map).contains(&action) {
        return Some(action);
    } else {
        println!("Not a valid action!");
        return get_player_action(&state, set_map);
    }
}

pub fn strategy_true_random(state: &GameState, set_map: &SetMap) -> Option<Action> {
    let mut actions = get_valid_actions(&state, set_map);
    actions.shuffle(&mut thread_rng());
    return actions.pop();
}

pub fn strategy_show_random(state: &GameState, set_map: &SetMap) -> Option<Action> {
    let mut actions = get_valid_actions(&state, set_map);
    actions.shuffle(&mut thread_rng());

    let show = actions.iter().find(|x| match x {
        Action::Show(_, _) => true,
        _ => false,
    });
    match show {
        Some(action) => return Some(*action),
        None => return actions.pop(),
    }
}

fn wl_pruning(state: &GameState, actions: &Vec<Action>) -> Vec<Action> {
    let mut pruned_actions: Vec<Action> = Vec::new();
    for action in actions {
        match state.take_action(action) {
            NewGameState::GameOver(scores) => {
                // If this action results in win, only return it
                if scores[0] == *scores.iter().max().unwrap() {
                    return vec![*action];
                }
            }
            NewGameState::Continue(_) => pruned_actions.push(*action),
        }
    }
    return pruned_actions;
}

pub fn strategy_show_wl_pruning(state: &GameState, set_map: &SetMap) -> Option<Action> {
    let mut all_actions = get_valid_actions(&state, set_map);
    all_actions.shuffle(&mut thread_rng());

    let mut actions = wl_pruning(&state, &all_actions);
    if actions.is_empty() {
        return all_actions.pop();
    }

    let show = actions.iter().find(|x| match x {
        Action::Show(_, _) => true,
        _ => false,
    });
    match show {
        Some(action) => return Some(*action),
        None => return actions.pop(),
    }
}

/// Returns minimum number of show actions required to empty hand
pub fn turns_to_empty(hand: &Vec<i32>, set_map: &SetMap) -> usize {
    // Iter through start and stops, then call recursively on self.
    // min gives None if iterator is empty (in this case the hand is empty
    (0..hand.len())
        .flat_map(|start| (start..hand.len()).map(move |stop| (start..stop + 1)))
        .map(|range| {
            let mut new_hand = hand.clone();
            let set: Vec<i32> = new_hand.drain(range).collect();
            (set, new_hand)
        })
        .filter(|(set, _)| set_map.contains_key(set))
        .map(|(_, new_hand)| turns_to_empty(&new_hand, &set_map) + 1)
        .min()
        .unwrap_or(0) // TODO: Cache!
}

pub fn evaluate_strategies(strategies: &Vec<Strategy>, n: usize) -> Vec<i32> {
    let n_strategies = strategies.len();
    let mut wins = vec![0; n_strategies];
    for _ in 0..n {
        match run(&strategies) {
            Ok(game_result) => {
                let max_score = *game_result.scores.iter().max().unwrap();
                for i in 0..n_strategies {
                    if game_result.scores[i] == max_score {
                        wins[i] += 1;
                    }
                }
            }
            Err(_) => {}
        }
    }
    return wins;
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

    #[test]
    fn test_set_map() {
        let set_map = generate_set_map();

        // Empty or non matching returns None
        assert_eq!(set_map.get(&Vec::new()), None);

        // Minimum set must score 1 (0 is empty set score)
        assert_eq!(set_map.get(&vec![0 as i32]), Some(&1));

        // Larger sets beat smaller sets
        assert!(set_map.get(&vec![1, 1, 1]).unwrap() > set_map.get(&vec![9, 9]).unwrap());

        // Flushes beat straights
        assert!(set_map.get(&vec![1, 1]).unwrap() > set_map.get(&vec![9, 8]).unwrap());
        assert!(set_map.get(&vec![4, 4, 4]).unwrap() > set_map.get(&vec![1, 2, 3]).unwrap());

        // Ascending == descending
        assert!(set_map.get(&vec![1, 2, 3]).unwrap() == set_map.get(&vec![3, 2, 1]).unwrap());
    }

    #[test]
    fn test_turns_to_empty() {
        let set_map = generate_set_map();

        //Trivial cases
        assert_eq!(turns_to_empty(&vec![], &set_map), 0);
        assert_eq!(turns_to_empty(&vec![0], &set_map), 1);
        assert_eq!(turns_to_empty(&vec![0, 1, 2], &set_map), 1);

        // Fiddly examples
        assert_eq!(turns_to_empty(&vec![0, 1, 0], &set_map), 2);
        assert_eq!(turns_to_empty(&vec![1, 3, 5], &set_map), 3);
        assert_eq!(turns_to_empty(&vec![1, 3, 1], &set_map), 2);
        assert_eq!(turns_to_empty(&vec![1, 3, 3, 1], &set_map), 2);
        assert_eq!(turns_to_empty(&vec![1, 3, 5, 7, 1], &set_map), 4);

        // Big hands
        assert_eq!(
            turns_to_empty(&vec![7, 3, 2, 1, 4, 7, 1, 2, 1], &set_map),
            5
        );
    }
}
