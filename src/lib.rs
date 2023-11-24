use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, VecDeque};
use std::fmt;

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
    pub hand: Vec<i32>,
    pub active: Set,
    pub active_owner: usize,
    pub scores: Vec<i32>,
    pub hand_sizes: Vec<usize>,
    pub scout_show: Vec<bool>,
}

enum NewGameState {
    Continue(GameState),
    GameOver(Vec<i32>),
}

/// Result of taking an action on a GameView. This may end the game, resulting in Win or Loss.
pub enum NewGameView {
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

    pub fn take_action(&self, action: &Action) -> NewGameView {
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

/// Strategy trait implements get_action method to generates an `Action` based on a `GameView`.
/// These can include user input, but are mostly computer players.
/// Returning `None` will halt the current game.

pub trait Strategy {
    fn get_action(&mut self, view: &GameView) -> Option<Action>;
}

pub struct GameResult {
    pub scores: Vec<i32>,
}

/// Run a single game of Scout. The length of `strategies` determines the number of players, and the
/// `Strategy` function each player uses.
///
/// Returns `GameResult` object containing final scores,
/// or in the case of runtime error, the `GameState` which lead to the error.
pub fn run(strategies: &mut Vec<Box<dyn Strategy>>) -> Result<GameResult, GameState> {
    let n_players = strategies.len();
    let mut game = GameState::new(n_players, true);

    loop {
        let action = strategies[game.turn].get_action(&game.as_view());
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
pub fn watch(
    strategies: &mut Vec<Box<dyn Strategy>>,
    show_hands: bool,
) -> Result<GameResult, GameState> {
    let n_players = strategies.len();
    let mut game = GameState::new(n_players, true);
    let mut round = 0;

    loop {
        if game.turn == 0 {
            println!("\nRound {}", round);
            if show_hands {
                println!("{}", game)
            };
            round += 1;
        }

        // Get action using strategy
        let action = strategies[game.turn].get_action(&game.as_view());
        match action {
            Some(action) => {
                println!("Active: {:?}", top_only(&game.active));
                println!("Player {} plays: {}", &game.turn, action);
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

/// To efficiently compare the value of sets, this hashmap is created.
pub type SetMap = HashMap<Vec<i32>, i32>;

/// Generate the default set hierarchy for the deck of 0-9 valued cards
pub fn default_set_map() -> SetMap {
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
pub fn get_valid_actions(view: &GameView, set_map: &SetMap) -> Vec<Action> {
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

    // To find all scout and show actions, iter through all valid scout actions, then generate
    // a new GameView and find all valid show actions
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
        let set_map = default_set_map();

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
        let set_map = default_set_map();
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
}
