use postflop_solver::*;

#[inline]
fn action_to_string(action: Action) -> String {
    match action {
        Action::Fold => "Fold:0".to_string(),
        Action::Check => "Check:0".to_string(),
        Action::Call => "Call:0".to_string(),
        Action::Bet(amount) => format!("Bet:{amount}"),
        Action::Raise(amount) => format!("Raise:{amount}"),
        Action::AllIn(amount) => format!("Allin:{amount}"),
        _ => unreachable!(),
    }
}

#[inline]
fn encode_action(action: Action) -> String {
    match action {
        Action::Fold => "F".to_string(),
        Action::Check => "X".to_string(),
        Action::Call => "C".to_string(),
        Action::Bet(amount) => format!("B{amount}"),
        Action::Raise(amount) => format!("R{amount}"),
        Action::AllIn(amount) => format!("A{amount}"),
        _ => unreachable!(),
    }
}

fn encode_line(line: &[Action]) -> String {
    let mut flag = 0;
    let mut encoded = String::new();

    if line.is_empty() {
        return "(Root)".to_string();
    }

    for &action in line {
        if !encoded.is_empty() {
            let delimiter = if flag == 2 { "|" } else { "-" };
            flag = if flag == 2 { 0 } else { flag };
            encoded.push_str(delimiter);
        }
        match action {
            Action::Check => flag += 1,
            Action::Call => flag = 2,
            _ => flag = 0,
        }
        encoded.push_str(&encode_action(action));
    }

    encoded
}

#[inline]
fn decode_action(action: &str) -> Action {
    match action {
        "F" => Action::Fold,
        "X" => Action::Check,
        "C" => Action::Call,
        _ => {
            let mut chars = action.chars();
            let first_char = chars.next().unwrap();
            let amount = chars.as_str().parse().unwrap();
            match first_char {
                'B' => Action::Bet(amount),
                'R' => Action::Raise(amount),
                'A' => Action::AllIn(amount),
                _ => unreachable!(),
            }
        }
    }
}

pub fn default_action_tree() -> ActionTree {
    let tree_config = TreeConfig {
        starting_pot: 1,
        effective_stack: 1,
        ..Default::default()
    };
    ActionTree::new(tree_config).unwrap()
}

#[allow(clippy::too_many_arguments)]
pub fn tree_new(
    tree_state: &mut ActionTree,
    board_len: i32,
    starting_pot: i32,
    effective_stack: i32,
    donk_option: bool,
    oop_flop_bet: String,
    oop_flop_raise: String,
    oop_turn_bet: String,
    oop_turn_raise: String,
    oop_turn_donk: String,
    oop_river_bet: String,
    oop_river_raise: String,
    oop_river_donk: String,
    ip_flop_bet: String,
    ip_flop_raise: String,
    ip_turn_bet: String,
    ip_turn_raise: String,
    ip_river_bet: String,
    ip_river_raise: String,
    add_allin_threshold: f64,
    force_allin_threshold: f64,
    merging_threshold: f64,
    added_lines: String,
    removed_lines: String,
) -> bool {
    let initial_state = match board_len {
        len if len <= 3 => BoardState::Flop,
        4 => BoardState::Turn,
        5 => BoardState::River,
        _ => panic!("Invalid board length"),
    };

    let config = TreeConfig {
        initial_state,
        starting_pot,
        effective_stack,
        rake_rate: 0.0,
        rake_cap: 0.0,
        flop_bet_sizes: [
            BetSizeOptions::try_from((oop_flop_bet.as_str(), oop_flop_raise.as_str())).unwrap(),
            BetSizeOptions::try_from((ip_flop_bet.as_str(), ip_flop_raise.as_str())).unwrap(),
        ],
        turn_bet_sizes: [
            BetSizeOptions::try_from((oop_turn_bet.as_str(), oop_turn_raise.as_str())).unwrap(),
            BetSizeOptions::try_from((ip_turn_bet.as_str(), ip_turn_raise.as_str())).unwrap(),
        ],
        river_bet_sizes: [
            BetSizeOptions::try_from((oop_river_bet.as_str(), oop_river_raise.as_str())).unwrap(),
            BetSizeOptions::try_from((ip_river_bet.as_str(), ip_river_raise.as_str())).unwrap(),
        ],
        turn_donk_sizes: match donk_option {
            false => None,
            true => DonkSizeOptions::try_from(oop_turn_donk.as_str()).ok(),
        },
        river_donk_sizes: match donk_option {
            false => None,
            true => DonkSizeOptions::try_from(oop_river_donk.as_str()).ok(),
        },
        add_allin_threshold,
        force_allin_threshold,
        merging_threshold,
    };

    let mut tree = ActionTree::new(config).unwrap();

    if !added_lines.is_empty() {
        for line in added_lines.split(',') {
            let line = line
                .split(&['-', '|'][..])
                .map(decode_action)
                .collect::<Vec<_>>();
            if tree.add_line(&line).is_err() {
                return false;
            }
        }
    }

    if !removed_lines.is_empty() {
        for line in removed_lines.split(',') {
            let line = line
                .split(&['-', '|'][..])
                .map(decode_action)
                .collect::<Vec<_>>();
            if tree.remove_line(&line).is_err() {
                return false;
            }
        }
    }

    *tree_state = tree;
    true
}

pub fn tree_added_lines(tree_state: &ActionTree) -> String {
    tree_state
        .added_lines()
        .iter()
        .map(|l| encode_line(l))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn tree_removed_lines(tree_state: &ActionTree) -> String {
    tree_state
        .removed_lines()
        .iter()
        .map(|l| encode_line(l))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn tree_invalid_terminals(tree_state: &ActionTree) -> String {
    tree_state
        .invalid_terminals()
        .iter()
        .map(|l| encode_line(l))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn tree_actions(tree_state: &ActionTree) -> Vec<String> {
    tree_state
        .available_actions()
        .iter()
        .cloned()
        .map(action_to_string)
        .collect()
}

pub fn tree_is_terminal_node(tree_state: &ActionTree) -> bool {
    tree_state.is_terminal_node()
}

pub fn tree_is_chance_node(tree_state: &ActionTree) -> bool {
    tree_state.is_chance_node()
}

pub fn tree_back_to_root(tree_state: &mut ActionTree) {
    tree_state.back_to_root();
}

pub fn tree_apply_history(tree_state: &mut ActionTree, line: Vec<String>) {
    let line = line
        .iter()
        .map(|l| decode_action(l.as_str()))
        .collect::<Vec<_>>();
    tree_state.apply_history(&line).unwrap();
}

pub fn tree_play(tree_state: &mut ActionTree, action: String) -> i32 {
    let action = decode_action(&action);
    let available_actions = tree_state.available_actions();
    if let Some(index) = available_actions.iter().position(|&a| a == action) {
        tree_state.play(action).unwrap();
        index as i32
    } else {
        -1
    }
}

pub fn tree_total_bet_amount(tree_state: &ActionTree) -> [i32; 2] {
    tree_state.total_bet_amount()
}

pub fn tree_add_bet_action(tree_state: &mut ActionTree, amount: i32, is_raise: bool) {
    let action = match is_raise {
        false => Action::Bet(amount),
        true => Action::Raise(amount),
    };
    tree_state.add_action(action).unwrap();
}

pub fn tree_remove_current_node(tree_state: &mut ActionTree) {
    tree_state.remove_current_node().unwrap();
}

pub fn tree_delete_added_line(tree_state: &mut ActionTree, line: String) {
    let line = line
        .split(&['-', '|'][..])
        .map(decode_action)
        .collect::<Vec<_>>();
    tree_state.remove_line(&line).unwrap();
}

pub fn tree_delete_removed_line(tree_state: &mut ActionTree, line: String) {
    let line = line
        .split(&['-', '|'][..])
        .map(decode_action)
        .collect::<Vec<_>>();
    tree_state.add_line(&line).unwrap();
}
