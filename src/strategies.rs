use rand::seq::SliceRandom;
use rand::thread_rng;
use scout_game::{get_valid_actions, Action, GameView, NewGameView, SetMap, Strategy};
use std::collections::HashMap;
use std::io;

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
pub struct GetPlayerAction {
    set_map: SetMap,
}

impl GetPlayerAction {
    pub fn new() -> GetPlayerAction {
        GetPlayerAction {
            set_map: scout_game::default_set_map(),
        }
    }
}

impl Strategy for GetPlayerAction {
    fn get_action(&mut self, view: &GameView) -> Option<Action> {
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
                let scout =
                    Action::Scout(split[1] == "1", split[2] == "1", split[3].parse().unwrap());

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

                // Players may enter "show 1 1" or "1 1" - best to just accept both
                let start: usize;
                let stop: usize;
                if show_split.len() == 2 {
                    start = show_split[0].parse().unwrap();
                    stop = show_split[1].parse().unwrap();
                } else {
                    start = show_split[1].parse().unwrap();
                    stop = show_split[2].parse().unwrap();
                }

                Action::ScoutShow(
                    split[1] == "1",
                    split[2] == "1",
                    split[3].parse().unwrap(),
                    start,
                    stop,
                )
            }
            "quit" => return None,
            _ => {
                println!("Input not accepted! Enter: scout, show, scoutshow, or quit");
                return self.get_action(&view);
            }
        };
        if get_valid_actions(&view, &self.set_map).contains(&action) {
            return Some(action);
        } else {
            println!("Not a valid action!");
            return self.get_action(&view);
        }
    }
}

/// Simple strategy which simply minimises the number of show turns required to
/// empty the current hand. This results in aggressive rush plays, and is especially
/// weak to mid-game large sets.
pub struct StrategyRush {
    set_map: SetMap,
    cache: HashMap<Vec<i32>, usize>,
}

impl StrategyRush {
    pub fn new() -> StrategyRush {
        StrategyRush {
            set_map: scout_game::default_set_map(),
            cache: HashMap::new(),
        }
    }
}

impl Strategy for StrategyRush {
    fn get_action(&mut self, view: &GameView) -> Option<Action> {
        let mut actions = get_valid_actions(&view, &self.set_map);
        actions.shuffle(&mut thread_rng());

        let mut cache = self.cache.clone();

        actions.sort_by_key(|action| match view.take_action(action) {
            NewGameView::Continue(new) => turns_to_empty(&new.hand, &self.set_map, &mut cache) + 1,
            NewGameView::Win => 0,
            NewGameView::Loss => 32,
        });

        self.cache = cache;
        return Some(actions[0]);
    }
}

// /// WIP - Strategy which considers a few metrics based on specified Weights and picks the highest score.
// /// See Strategy TODO for related structural issue here.
// pub struct StrategyWeighted {
//     set_map: SetMap,
//     cache: HashMap<Vec<i32>, usize>,
//     scout: i32,
//     show: i32,
//     scoutshow: i32,
//     turns_to_empty: i32,
// }
// impl Strategy for StrategyWeighted {
//     fn get_action(&self, view: &GameView) -> Option<Action> {
//         let actions = get_valid_actions(&view, &self.set_map);

//         return actions
//             .iter()
//             .max_by_key(|action| {
//                 (match view.take_action(action) {
//                     NewGameView::Win => 100,
//                     NewGameView::Loss => 0,
//                     NewGameView::Continue(new_view) => {
//                         &self.turns_to_empty
//                             * turns_to_empty(&new_view.hand, &self.set_map, &mut self.cache) as i32
//                     }
//                 }) * (match action {
//                     Action::Scout(_, _, _) => self.scout,
//                     Action::Show(_, _) => self.show,
//                     Action::ScoutShow(_, _, _, _, _) => self.scoutshow,
//                 })
//             })
//             .copied();
//     }
// }

#[test]
fn test_turns_to_empty() {
    let set_map = scout_game::default_set_map();
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
