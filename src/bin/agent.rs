use std::error::Error;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use muehle_game::{calculate_next_position, MILLS};
use muehle_game::datastructures::movement::Move;
use muehle_game::datastructures::phase::GamePhase;
use muehle_game::datastructures::player::Player;
use muehle_game::datastructures::point::Point;
use muehle_game::datastructures::position::Position;

struct SearchContext {
    start: Instant,
    timeout: AtomicBool
}

impl SearchContext {
    fn new() -> Self {
        SearchContext {
            start: Instant::now(),
            timeout: AtomicBool::new(false)
        }
    }

    #[inline(always)]
    fn check_time(&self) -> bool {
        if self.start.elapsed() >= Duration::from_millis(915) {
            self.timeout.store(true, Ordering::Relaxed);
            return true;
        }
        false
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut position_count: HashMap<Position, u8> = HashMap::new();

    loop {
        let (phase, position) = parse_input();

        *position_count.entry(position.clone()).or_insert(0) += 1;

        let best_move = minimax(&position, phase, 20, &mut position_count);
        dbg!(best_move);
        if let Some(best_move) = best_move {
            println!("{}", best_move);
        } else {
            println!("NONE");
        }
    }

}

fn pick_best_move_first(moves: &mut Vec<Move>, hint: Option<Move>) {
    let mut head = 0;

    if let Some(h) = hint {
        if let Some(pos) = moves.iter().position(|&m| m == h) {
            moves.swap(0, pos);
            head = 1;
        }
    }
    let len = moves.len();
    for i in head..len {
        if is_kill(moves[i]) {
            moves.swap(head, i);
            head += 1;
        }
    }
}

fn is_kill(move_action: Move) -> bool {
    matches!(move_action, Move::Place { kill: Some(_), .. } | Move::Move { kill: Some(_), .. })
}


fn minimax(position: &Position, game_phase: GamePhase, max_depth: u32, position_history: &mut HashMap<Position, u8>) -> Option<Move> {
    let context = SearchContext::new();
    let mut best_move = None;

    let agent_color = position.active_player();
    for current_depth in 1..=max_depth {
        let (value, mv) = negamax(position, game_phase, &context, current_depth, -1_000_000_000, 1_000_000_000, best_move, agent_color, agent_color, position_history);

        let time_used = context.start.elapsed().as_millis();
        if context.timeout.load(Ordering::Relaxed) {
            eprintln!(">> Tiefe {} ABGEBROCHEN bei {}ms.", current_depth, time_used);
            break;
        }
        if let Some(m) = mv {
            best_move = Some(m);
            eprintln!(">> Tiefe {} fertig in {}ms. Move: {:?}", current_depth, time_used, m);
        }

        if value > 999_000 {
            eprintln!("Sieg in Tiefe {} gefunden", current_depth);
            break;
        }
    }
    best_move
}

fn negamax(
    position: &Position,
    game_phase: GamePhase,
    ctx: &SearchContext,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    move_hint: Option<Move>,
    agent_color: Player,
    current_search_color: Player,
    position_history: &mut HashMap<Position, u8>
) -> (i32, Option<Move>) {
    if ctx.check_time() || ctx.timeout.load(Ordering::Relaxed) {
        return (0, None);
    }

    let count = *position_history.get(position).unwrap_or(&0);

    if count >= 2 {
        return (-500, None);
    }

    position_history.insert(position.clone(), count + 1);

    if depth == 0 {
        let score = rate_position(position, game_phase, agent_color);
        position_history.insert(position.clone(), count);
        return if current_search_color == agent_color {
            (score, None)
        } else {
            (-score, None)
        }
    }

    let mut value: i32 = i32::MIN;
    let mut best_move: Option<Move> = None;
    let mut vec: Vec<Move> = position.enumerate_moves().collect();
    if vec.is_empty() {
        let score = rate_position(position, game_phase, agent_color);
        position_history.insert(position.clone(), count);
        return if current_search_color == agent_color {
            (score, None)
        } else {
            (-score, None)
        }
    }

    pick_best_move_first(&mut vec, move_hint);

    let next_color = current_search_color.opposite();

    for move_action in vec {
        let applied_move = calculate_next_position(position, move_action).unwrap();
        let (neg_val, _) = negamax(&applied_move, game_phase, ctx, depth - 1, -beta, -alpha, None, agent_color, next_color, position_history);
        let neg_val = -neg_val;

        if ctx.timeout.load(Ordering::Relaxed) {
            position_history.insert(position.clone(), count);
            return (0, None);
        }

        if neg_val > value {
            value = neg_val;
            best_move = Some(move_action);
        }

        if value >= beta {
            break;
        }
        alpha = alpha.max(value);

    }
    position_history.insert(position.clone(), count);
    (value, best_move)
}

fn rate_position(position: &Position, game_phase: GamePhase, agent_color: Player) -> i32 {
    let mut position_value: i32 = 0;

    let own_stones = position.get_piece_count_for_player(agent_color) as i32;
    let opponent_stones = position.get_piece_count_for_player(agent_color.opposite()) as i32;

    if let GamePhase::Moving = game_phase {
        if own_stones < 3 { return -10_000_000; }
        if opponent_stones < 3 { return 10_000_000; }
        if position.enumerate_moves().next().is_none() { return -10_000_000; }
    }

    position_value += (own_stones - opponent_stones) * 50000;

    for line in MILLS.iter() {
        let mut own_stones = 0;
        let mut opponent_stones = 0;
        let mut empty_field = 0;

        for &point in line.iter() {
            match position.get_piece_at_position(point) {
                Point::Occupied(player) if player == agent_color => own_stones += 1,
                Point::Occupied(_) => opponent_stones += 1,
                Point::Empty => empty_field += 1,
            }
        }

        if own_stones == 3 {
            position_value += 10000;
        }
        if own_stones == 2 && empty_field == 1 {
            position_value += 9500;
        }
        if own_stones == 1 && empty_field == 2 {
            position_value += 1000;
        }
        if opponent_stones == 2 && empty_field == 1 {
            position_value -= 20000;
        }
        if opponent_stones == 3 {
            position_value -= 9000;
        }
        if own_stones == 1 && opponent_stones == 1 {
            position_value -= 1000;
        }

    }

    let mut opponent_position = position.clone();
    opponent_position.switch_player();
    position_value += ((position.enumerate_moves().count() as i32) - (opponent_position.enumerate_moves().count() as i32)) * 20;

    position_value
}

fn parse_input() -> (GamePhase, Position) {
    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let mut clean_input = input.split_whitespace();

    let (phase, round) = match clean_input.next().unwrap() {
        "P" => (GamePhase::Placing, Some(clean_input.next().unwrap().parse::<u8>().unwrap())),
        "M" => (GamePhase::Moving, None),
        _ => panic!("Unknown phase")
    };

    let pos_str = clean_input.next().unwrap();
    let mut position: Position = pos_str.parse().unwrap();

    // Runde n: Weiß hat n-1 gesetzt, wenn Weiß dran ist, sonst schon n.
    let pieces_to_place = match round {
        Some(n) => {
            let left = 10u8.saturating_sub(n);
            let white_left = if position.active_player() == Player::White { left } else { left.saturating_sub(1) };
            (white_left, left)
        }
        None => (0, 0)
    };
    position.set_pieces_to_place(pieces_to_place);

    (phase, position)
}