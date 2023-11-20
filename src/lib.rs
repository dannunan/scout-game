use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, VecDeque};
use std::{fmt, io};

#[derive(Debug, Clone)]
/// A card, this stores two values, however only the first is "active".
/// Implements `flip()`, a convenience method which simply flips the two values.
pub struct Card(i32, i32);

impl Card {
    fn flip(&self) -> Card {
        Card(self.1, self.0)
    }
}

/// A set of cards. This can represent a hand or a set.
type Set = VecDeque<Card>;

/// Each player has a hand, score, and their "Scout show" move.
#[derive(Debug, Clone)]
pub struct Player {
    hand: Set,
    score: i32,
    scout_show: bool,
}

/// Player actions. These are Scout, Show and ScoutShow, which each take
/// different parameters.
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
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

/// Stores information about the whole game.
/// This implements game logic to process actions and check scores.
/// It is also responsible for generating `GameView` objects from the perspective of
/// the current player (`self.turn`)
#[derive(Debug, Default)]
pub struct GameState {
    players: VecDeque<Player>,
    game_size: usize,
    active: Set,
    active_owner: usize,
    turn: usize,
}

/// View from perspective of single player. This is rotated, so vectors such as
/// `score` may not align with the "true" indexes in `GameState`
///
/// e.g. a `GameView` from the perspective of player 2 will store player 2's data in
/// index 0, and player 3's data in index 1.
#[derive(Clone)]
pub struct GameView {
    hand: Vec<i32>,
    active: Set,
    active_owner: usize,
    scores: Vec<i32>,
    hand_sizes: Vec<usize>,
    scout_show: Vec<bool>,
}

enum NewGameState {
    Continue(GameState),
    GameOver(Vec<i32>),
}

enum NewGameView {
    Continue(GameView),
    Win,
    Loss,
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
            Self::ScoutShow(left, flip, insert, start, stop) => {
                write!(
                    f,
                    "Scout and show! l:{}, f:{}, i:{}, {} to {}",
                    left, flip, insert, start, stop
                )
            }
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Score: {}, Hand: {:?}", self.score, top_only(&self.hand))
    }
}

impl Default for Player {
    fn default() -> Self {
        Player {
            hand: Default::default(),
            score: Default::default(),
            scout_show: true,
        }
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut hands: String = "Hands:\n".to_owned();
        for player in &self.players {
            hands.push_str(&format!("{}\n", player))
        }
        write!(f, "{}", hands)
    }
}

impl fmt::Display for GameView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string: String = Default::default();
        string.push_str("\nActive Set:\n");

        for card in &self.active {
            string.push_str(&format!("{}  |", card.0))
        }
        string.push_str("\n");
        for card in &self.active {
            string.push_str(&format!("  {}|", card.1))
        }
        string.push_str("\n");

        string.push_str(&format!("\nPoints: {} Hand:", self.scores[0]));
        string.push_str(&format!("{:?}", self.hand));
        write!(f, "{}", string)
    }
}

impl GameState {
    fn new(n: usize, shuffle: bool) -> Self {
        let mut game = GameState {
            active: Set::new(),
            game_size: n,
            players: VecDeque::from(vec![Default::default(); n]),
            active_owner: 0,
            turn: 0,
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
        let turn = self.turn;

        let mut card: Card;
        if left {
            card = active.pop_front().unwrap();
        } else {
            card = active.pop_back().unwrap();
        }
        if flip {
            card = card.flip();
        }
        players[turn].hand.insert(index, card);
        players[active_owner].score += 1;

        GameState {
            players,
            game_size,
            active,
            active_owner,
            turn,
        }
    }

    fn show(&self, start: usize, stop: usize) -> GameState {
        let mut players = self.players.clone();
        let game_size = self.game_size;
        let mut active = self.active.clone();
        let active_owner = self.turn;
        let turn = self.turn;

        players[turn].score += active.len() as i32;
        active.clear();
        for _ in start..stop + 1 {
            active.push_back(players[turn].hand.remove(start).unwrap())
        }

        GameState {
            players,
            game_size,
            active,
            active_owner,
            turn,
        }
    }

    fn take_action(&self, action: &Action) -> NewGameState {
        let mut state: GameState;
        match action {
            Action::Scout(left, flip, index) => state = self.scout(*left, *flip, *index),
            Action::Show(start, stop) => state = self.show(*start, *stop),
            Action::ScoutShow(left, flip, index, start, stop) => {
                state = self.scout(*left, *flip, *index).show(*start, *stop);
                state.players[self.turn].scout_show = false;
            }
        };

        // Round ends if current players hand is empty
        if state.players[state.turn].hand.len() == 0 {
            return NewGameState::GameOver(
                state
                    .players
                    .iter()
                    .map(|p| p.score - p.hand.len() as i32)
                    .collect(),
            );
        }
        // Progress turn marker
        state.turn = (state.turn + 1) % state.game_size;

        // Round ends if active_owner is next player
        if state.active_owner == state.turn {
            // The next player isn't penalised for their hand size -
            // offset this players points by their hand size then count normally
            state.players[state.turn].score += state.players[state.turn].hand.len() as i32;
            return NewGameState::GameOver(
                state
                    .players
                    .iter()
                    .map(|p| p.score - p.hand.len() as i32)
                    .collect(),
            );
        } else {
            return NewGameState::Continue(state);
        }
    }

    fn as_view(&self) -> GameView {
        // Shuffle players left
        // For the view of player 1, player 4 is indexed 3
        let mut players = self.players.clone();
        players.rotate_left(self.turn);
        GameView {
            hand: top_only(&players[0].hand),
            active: self.active.clone(),
            active_owner: (self.active_owner + self.game_size - self.turn) % self.game_size,
            scores: players.iter().map(|p| p.score).collect(),
            hand_sizes: players.iter().map(|p| p.hand.len()).collect(),
            scout_show: players.iter().map(|p| p.scout_show).collect(),
        }
    }
}

impl GameView {
    fn scout(&self, left: bool, flip: bool, index: usize) -> GameView {
        let mut hand = self.hand.clone();
        let mut active = self.active.clone();
        let active_owner = self.active_owner;
        let mut scores = self.scores.clone();
        let mut hand_sizes = self.hand_sizes.clone();
        let scout_show = self.scout_show.clone();

        let card: Card;
        if left {
            card = active.pop_front().unwrap();
        } else {
            card = active.pop_back().unwrap();
        }
        if flip {
            hand.insert(index, card.1);
        } else {
            hand.insert(index, card.0);
        }
        hand_sizes[0] += 1;
        scores[active_owner] += 1;

        GameView {
            hand,
            active,
            active_owner,
            scores,
            hand_sizes,
            scout_show,
        }
    }

    fn show(&self, start: usize, stop: usize) -> GameView {
        let mut hand = self.hand.clone();
        let mut active = self.active.clone();
        let active_owner = 0;
        let mut scores = self.scores.clone();
        let hand_sizes = self.hand_sizes.clone();
        let scout_show = self.scout_show.clone();

        scores[0] += active.len() as i32;
        active.clear();
        for _ in start..stop + 1 {
            active.push_back(Card(hand.remove(start), 0))
        }

        GameView {
            hand,
            active,
            active_owner,
            scores,
            hand_sizes,
            scout_show,
        }
    }

    fn take_action(&self, action: &Action) -> NewGameView {
        let mut view: GameView;
        match action {
            Action::Scout(left, flip, index) => view = self.scout(*left, *flip, *index),
            Action::Show(start, stop) => view = self.show(*start, *stop),
            Action::ScoutShow(left, flip, index, start, stop) => {
                view = self.scout(*left, *flip, *index).show(*start, *stop);
                view.scout_show[0] = false;
            }
        };

        // Round ends if current players hand is empty
        if view.hand.len() == 0 {
            let mut final_scores = Vec::new();
            for i in 0..view.scores.len() {
                final_scores.push(view.scores[i] - view.hand_sizes[i] as i32);
            }
            if final_scores[0] == *final_scores.iter().max().unwrap() {
                return NewGameView::Win;
            } else {
                return NewGameView::Loss;
            };
        // Round ends if active owner is next player (1)
        } else if view.active_owner == 1 {
            // The next player isn't penalised for their hand size -
            // offset this players points by their hand size then count normally
            view.scores[1] += view.hand.len() as i32;
            let mut final_scores = Vec::new();
            for i in 0..view.scores.len() {
                final_scores.push(view.scores[i] - view.hand_sizes[i] as i32);
            }
            if final_scores[0] == *final_scores.iter().max().unwrap() {
                return NewGameView::Win;
            } else {
                return NewGameView::Loss;
            };
        } else {
            return NewGameView::Continue(view);
        }
    }
}

/// Strategies are functions which generate an `Action` based on a `GameView`.
/// These can include user input, but are mostly computer players.
/// Returning `None` will halt the current game.
/// `SetMap` is a HashMap of set values - this is static for the duration of a game.
///
/// TODO:
/// Possibly this whole structure needs some thought, currently strategies are stateless,
/// which limits caching to a single GameState. This also makes dynamically training strategies
/// difficult. This should probably be a struct with a get_action method.
pub type Strategy = fn(&GameView, &SetMap) -> Option<Action>;

pub struct GameResult {
    pub scores: Vec<i32>,
}

/// Run a single game of Scout. The length of `strategies` determines the number of players, and the
/// `Strategy` function each player uses.
///
/// Returns `GameResult` object containing final scores,
/// or in the case of runtime error, the `GameState` which lead to the error.
pub fn run(strategies: &Vec<Strategy>) -> Result<GameResult, GameState> {
    let set_map = generate_set_map();
    let n_players = strategies.len();
    let mut game = GameState::new(n_players, true);

    loop {
        let action = strategies[game.turn](&game.as_view(), &set_map);
        match action {
            Some(action) => {
                match game.take_action(&action) {
                    NewGameState::Continue(new) => game = new,
                    NewGameState::GameOver(scores) => {
                        return Ok(GameResult { scores });
                    }
                };
            }
            None => {
                return Err(game);
            }
        }
    }
}

/// Watch a single game of Scout. The length of `strategies` determines the number of players, and the
/// `Strategy` function each player uses.
///
/// Returns `GameResult` object containing final scores,
/// or in the case of runtime error, the `GameState` which lead to the error.
///
/// This function copies `scout::run` but has more side-effects, primarily for debugging.
/// TODO: this gives too much information for a human player, but the amount of info `run` can give
/// is limited as it only has access to a GameView - this function should serve this role, and some
/// effects should be removed from `get_player_action`
pub fn watch(strategies: &Vec<Strategy>) -> Result<GameResult, GameState> {
    let set_map = generate_set_map();
    let n_players = strategies.len();
    let mut game = GameState::new(n_players, true);
    let mut round = 0;

    loop {
        if game.turn == 0 {
            println!("\nRound {}", round);
            println!("{}", game);
            round += 1;
        }

        // Get action using strategy
        let action = strategies[game.turn](&game.as_view(), &set_map);
        match action {
            Some(action) => {
                println!("{:?}", top_only(&game.active));
                println!("Player {}: {}", &game.turn, action);
                match game.take_action(&action) {
                    NewGameState::Continue(new) => game = new,
                    NewGameState::GameOver(scores) => {
                        return Ok(GameResult { scores });
                    }
                };
            }
            None => {
                return Err(game);
            }
        }
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

/// To efficiently compare the value of sets, this hashmap is created. In practice this HashMap
/// is const, however due to const limitations this currently is created by `scout::run`.
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
    set.iter().map(|card| card.0).collect()
}

/// Return all valid Actions for `view`. For a large hand this will include approximately:
/// - 40 Scout actions
/// - 20 Show actions
/// - 600 ScoutShow actions
///
/// These numbers decrease rapidly as the game progresses - particularly once the ScoutShow action
/// is used for the round.
fn get_valid_actions(view: &GameView, set_map: &SetMap) -> Vec<Action> {
    let mut actions = Vec::new();

    // Scout actions
    if !view.active.is_empty() {
        for i in 0..view.hand.len() + 1 {
            for (left, flip) in [(false, false), (false, true), (true, false), (true, true)] {
                actions.push(Action::Scout(left, flip, i));
            }
        }
    }

    // Show actions
    let active_set_score = set_map.get(&top_only(&view.active)).unwrap_or(&0);
    let hand = &view.hand;
    for start in 0..hand.len() {
        for stop in start..hand.len() {
            if let Some(score) = set_map.get(&hand[start..stop + 1]) {
                if *score > *active_set_score {
                    actions.push(Action::Show(start, stop))
                }
            }
        }
    }

    // Scout and show actions
    if !view.scout_show[0] | view.active.is_empty() {
        return actions;
    }

    let mut new_hand: Vec<i32>;
    let mut scout_card: Card;
    let mut new_active: VecDeque<Card>;
    for i in 0..view.hand.len() + 1 {
        for left in [true, false] {
            for flip in [true, false] {
                new_active = view.active.clone();
                match left {
                    true => scout_card = new_active.pop_front().unwrap(),
                    false => scout_card = new_active.pop_back().unwrap(),
                };

                new_hand = hand.clone();
                match flip {
                    true => new_hand.insert(i, scout_card.1),
                    false => new_hand.insert(i, scout_card.0),
                }

                let new_active_set_score = set_map.get(&top_only(&new_active)).unwrap_or(&0);
                for start in 0..new_hand.len() {
                    for stop in start..new_hand.len() {
                        if let Some(score) = set_map.get(&new_hand[start..stop + 1]) {
                            if *score > *new_active_set_score {
                                actions.push(Action::ScoutShow(left, flip, i, start, stop))
                            }
                        }
                    }
                }
            }
        }
    }

    return actions;
}

/// Strategy which requests user for input
/// When prompted for an action, enter one of the following actions:
/// - `scout [left] [flip] [index]`
/// - `show [start] [stop]`
/// - `scoutshow [left] [flip] [index]`
/// - `quit`
///
/// All arguments should be numeric (1 representing `true`).
///
/// The **scout** action has arguments: `left` for which side of the active set to scout,
/// `flip` if the card is to be flipped, and the `index` to insert the card at.
///
/// The **show** action has arguments `start` and `stop`, which are the inclusive bounds of
/// the set to show. A single card can be played by repeating e.g. `show 2 2`.
///
/// The final action, **scoutshow**, is simply the above actions combined. You should first
/// enter arguments for the scout step, then you will be presented with a new view and can input a show action.
///
/// Entering **quit** will cause the game to halt. This will print a debug view of the `GameState` before exiting.
pub fn get_player_action(view: &GameView, set_map: &SetMap) -> Option<Action> {
    // Print some info
    println!("{}", view);
    let indexes: Vec<usize> = (0..view.hand.len()).collect();
    println!("       Indexes:{:?}\n", indexes);

    let mut input = String::new();
    println!("\nSelect action:");
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let split: Vec<&str> = input.trim().split(" ").collect();
    let action = match split[0] {
        "scout" => Action::Scout(split[1] == "1", split[2] == "1", split[3].parse().unwrap()),
        "show" => Action::Show(split[1].parse().unwrap(), split[2].parse().unwrap()),
        "scoutshow" => {
            let scout = Action::Scout(split[1] == "1", split[2] == "1", split[3].parse().unwrap());

            // Scout and show should never end round - halt if this happens
            // TODO: create modified copy of view to prevent scout round end condition
            let scout_view = match view.take_action(&scout) {
                NewGameView::Win => return None,
                NewGameView::Loss => return None,
                NewGameView::Continue(view) => view,
            };

            // Show player the resulting GameView after scouting
            println!("{}", scout_view);
            let indexes: Vec<usize> = (0..scout_view.hand.len()).collect();
            println!("       Indexes:{:?}\n", indexes);

            // Get show component
            let mut show_input = String::new();
            println!("\nSelect show action (finish scoutshow):");
            io::stdin()
                .read_line(&mut show_input)
                .expect("Failed to read line");
            let show_split: Vec<&str> = show_input.trim().split(" ").collect();
            Action::ScoutShow(
                split[1] == "1",
                split[2] == "1",
                split[3].parse().unwrap(),
                show_split[0].parse().unwrap(),
                show_split[1].parse().unwrap(),
            )
        }
        // TODO: Add Scoutshow input - this needs to preview hand after show, and give escape
        // option (in which case a normal Show option is returned)
        "quit" => return None,
        _ => {
            println!("Input not accepted! Enter: scout, show, scoutshow, or quit");
            return get_player_action(&view, set_map);
        }
    };
    if get_valid_actions(&view, set_map).contains(&action) {
        return Some(action);
    } else {
        println!("Not a valid action!");
        return get_player_action(&view, set_map);
    }
}

/// Simple strategy which simply minimises the number of show turns required to
/// empty the current hand. This results in aggressive rush plays, and is especially
/// weak to mid-game large sets.
pub fn strategy_rush(view: &GameView, set_map: &SetMap) -> Option<Action> {
    let mut actions = get_valid_actions(&view, set_map);
    actions.shuffle(&mut thread_rng());

    let mut cache = HashMap::new();

    actions.sort_by_key(|action| match view.take_action(action) {
        NewGameView::Continue(new) => turns_to_empty(&new.hand, &set_map, &mut cache) + 1,
        NewGameView::Win => 0,
        NewGameView::Loss => 32,
    });
    return Some(actions[0]);
}

pub struct Weights {
    scout: i32,
    show: i32,
    scoutshow: i32,
    turns_to_empty: i32,
}

/// WIP - Strategy which considers a few metrics based on specified Weights and picks the highest score.
/// See Strategy TODO for related structural issue here.
pub fn strategy_weighted(view: &GameView, set_map: &SetMap, weights: Weights) -> Option<Action> {
    let actions = get_valid_actions(&view, set_map);
    let mut cache = HashMap::new();

    return actions
        .iter()
        .max_by_key(|action| {
            (match view.take_action(action) {
                NewGameView::Win => 100,
                NewGameView::Loss => 0,
                NewGameView::Continue(new_view) => {
                    weights.turns_to_empty
                        * turns_to_empty(&new_view.hand, &set_map, &mut cache) as i32
                }
            }) * (match action {
                Action::Scout(_, _, _) => weights.scout,
                Action::Show(_, _) => weights.show,
                Action::ScoutShow(_, _, _, _, _) => weights.scoutshow,
            })
        })
        .copied();
}

/// Returns minimum number of show actions required to empty hand.
/// This iterates through all possible sets, checks validity against `set_map`,
/// then evaluates remaining hand recursively.
fn turns_to_empty(
    hand: &Vec<i32>,
    set_map: &SetMap,
    cache: &mut HashMap<Vec<i32>, usize>,
) -> usize {
    if set_map.contains_key(hand) {
        return 1;
    }

    let turns = match cache.get(hand) {
        Some(n) => return *n,
        None => (0..hand.len())
            .flat_map(|start| (start..hand.len()).map(move |stop| (start..stop + 1)))
            .map(|range| {
                let mut new_hand = hand.clone();
                let set: Vec<i32> = new_hand.drain(range).collect();
                (set, new_hand)
            })
            .filter(|(set, _)| set_map.contains_key(set))
            .map(
                |(_, new_hand)| match turns_to_empty(&new_hand, &set_map, cache) {
                    1 => {
                        cache.insert(hand.clone(), 2);
                        return 2; // Return early
                    }
                    x => x + 1,
                },
            )
            .min()
            .unwrap(),
    };

    // Cache!
    cache.insert(hand.clone(), turns);
    return turns;
}

/// Convenience function for bulk running `n` games.
///
/// Returns number of wins for each strategy.
/// Drawing for 1st place counts as a win, so the total may exceed the number of games.
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
    use std::collections::HashSet;

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
    fn test_get_valid_actions() {
        let set_map = generate_set_map();
        let base_view = GameView {
            hand: Vec::new(),
            active: VecDeque::new(),
            active_owner: 3,
            hand_sizes: vec![1, 1, 1, 1],
            scores: vec![0, 0, 0, 0],
            scout_show: vec![false, false, false, false],
        };

        // Test basic show cases
        let mut view = base_view.clone();
        view.hand.push(0); // hand: [0]
        let actions: HashSet<Action> = get_valid_actions(&view, &set_map).iter().copied().collect();
        assert_eq!(actions, HashSet::from_iter([Action::Show(0, 0)]));
        view.hand.push(0); // hand: [0, 0]
        let actions: HashSet<Action> = get_valid_actions(&view, &set_map).iter().copied().collect();
        assert_eq!(
            actions,
            HashSet::from_iter([Action::Show(0, 0), Action::Show(0, 1), Action::Show(1, 1),])
        );

        // Test basic scout cases
        let mut view = base_view.clone();
        view.hand.push(0); // hand: [0]
        view.active.push_back(Card(1, 1)); // active: [1]
        let actions: HashSet<Action> = get_valid_actions(&view, &set_map).iter().copied().collect();
        assert_eq!(
            actions,
            HashSet::from_iter([
                Action::Scout(false, false, 0),
                Action::Scout(false, true, 0),
                Action::Scout(true, false, 0),
                Action::Scout(true, true, 0),
                Action::Scout(false, false, 1),
                Action::Scout(false, true, 1),
                Action::Scout(true, false, 1),
                Action::Scout(true, true, 1),
            ])
        );

        // Test more complex scout show case
        let mut view = base_view.clone();
        view.hand.push(0); // hand: [0]
        view.active.push_back(Card(3, 0)); // this 0 can be used with scoutshow
        view.active.push_back(Card(3, 3)); // active: [3, 3]
        view.scout_show[0] = true;
        let actions: HashSet<Action> = get_valid_actions(&view, &set_map).iter().copied().collect();
        assert_eq!(
            actions,
            HashSet::from_iter([
                Action::Scout(false, false, 0),
                Action::Scout(false, true, 0),
                Action::Scout(true, false, 0),
                Action::Scout(true, true, 0),
                Action::Scout(false, false, 1),
                Action::Scout(false, true, 1),
                Action::Scout(true, false, 1),
                Action::Scout(true, true, 1),
                Action::ScoutShow(true, true, 0, 0, 1),
                Action::ScoutShow(true, true, 1, 0, 1),
            ])
        );
    }

    #[test]
    fn test_turns_to_empty() {
        let set_map = generate_set_map();
        let mut cache: HashMap<Vec<i32>, usize> = HashMap::new();

        // Trivial cases
        assert_eq!(turns_to_empty(&vec![0], &set_map, &mut cache), 1);
        assert_eq!(turns_to_empty(&vec![0, 1, 2], &set_map, &mut cache), 1);

        // Fiddly examples
        assert_eq!(turns_to_empty(&vec![0, 1, 0], &set_map, &mut cache), 2);
        assert_eq!(turns_to_empty(&vec![1, 3, 5], &set_map, &mut cache), 3);
        assert_eq!(turns_to_empty(&vec![1, 3, 1], &set_map, &mut cache), 2);
        assert_eq!(turns_to_empty(&vec![1, 3, 3, 1], &set_map, &mut cache), 2);
        assert_eq!(
            turns_to_empty(&vec![1, 3, 5, 7, 1], &set_map, &mut cache),
            4
        );

        // Big hands
        assert_eq!(
            turns_to_empty(&vec![7, 3, 2, 1, 4, 7, 1, 2, 1], &set_map, &mut cache),
            5
        );
    }
}
