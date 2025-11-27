use muehle_game::{calculate_next_position, GamePhase, Move, Point, Position};
use std::error::Error;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub const MILL_LINES: [[u8; 3]; 16] = [
    [7, 0, 1],
    [1, 2, 3],
    [3, 4, 5],
    [5, 6, 7],

    [15, 8, 9],
    [9, 10, 11],
    [11, 12, 13],
    [13, 14, 15],

    [23, 16, 17],
    [17, 18, 19],
    [19, 20, 21],
    [21, 22, 23],

    [0, 8, 16],
    [2, 10, 18],
    [4, 12, 20],
    [6, 14, 22],
];

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
        if self.start.elapsed() >= Duration::from_millis(950) {
            self.timeout.store(true, Ordering::Relaxed);
            return true;
        }
        false
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        let (phase, position) = parse_input();

        let best_move = minimax(&position, phase, 20);
        dbg!(best_move);

        if let Some(best_move) = best_move {
            match best_move {
                Move::Place {position, kill} => {
                    if let Some(remove) = kill {
                        println!("P {} T {}", position, remove);
                    } else {
                        println!("P {}", position);
                    }
                }
                Move::Move {from_position, to_position, kill} => {
                    if let Some(remove) = kill {
                        println!("M {} {} T {}", from_position, to_position, remove);
                    } else {
                        println!("M {} {}", from_position, to_position);
                    }
                }
            }
        } else {
            println!("NONE");
        }
    }

}

/*
fn pick_best_move_first(moves: &mut impl Iterator<Item=Move>, hint: Option<Move>) {
    let mut head = 0;

    if let Some(h) = hint {
        if let Some(pos) = moves.position(|m| m == h) {
            moves.swap(0, pos);
            head = 1;
        }
    }
    let len = moves.len();
    for i in head..len {
        if moves[i].take.is_some() {
            moves.swap(head, i);
            head += 1;
        }
    }
}*/


fn minimax(position: &Position, game_phase: GamePhase, max_depth: u32) -> Option<Move> {
    let context = SearchContext::new();
    let mut best_move = None;

    for current_depth in 1..=max_depth {
        let (_, mv) = negamax(position, game_phase, &context, current_depth, -1_000_000_000, 1_000_000_000, best_move);

        //dbg
        let time_used = context.start.elapsed().as_millis();
        if context.timeout.load(Ordering::Relaxed) {
            eprintln!(">> Tiefe {} ABGEBROCHEN bei {}ms.", current_depth, time_used);
            break;
        }
        if let Some(m) = mv {
            best_move = Some(m);
            eprintln!(">> Tiefe {} fertig in {}ms. Move: {:?}", current_depth, time_used, m);
        }
        //dbg
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
    move_hint: Option<Move>
) -> (i32, Option<Move>) {
    if ctx.check_time() || ctx.timeout.load(Ordering::Relaxed) {
        return (0, None);
    }

    if depth == 0 {
        return (rate_position(position, game_phase), None);
    }

    let mut value: i32 = i32::MIN;
    let mut best_move: Option<Move> = None;
    let mut possible_moves = position.enumerate_moves().peekable();
    if possible_moves.peek().is_none() {
        return (rate_position(position, game_phase), None);
    }

    //pick_best_move_first(&mut possible_moves, move_hint);

    for move_action in possible_moves {
        let applied_move = calculate_next_position(position, move_action).unwrap();
        let (neg_val, _) = negamax(&applied_move, game_phase, ctx, depth - 1, -beta, -alpha, None);
        let neg_val = -neg_val;

        if ctx.timeout.load(Ordering::Relaxed) {
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
    (value, best_move)

}

fn rate_position(position: &Position, game_phase: GamePhase) -> i32 {
    let mut position_value: i32 = 0;
    let active_player = position.active_player();
    let opponent_player = active_player.opposite();

    let mut own_stones: i32 = 0;
    let mut opponent_stones: i32 = 0;
    own_stones += position.get_piece_count_for_player(active_player) as i32;
    opponent_stones += position.get_piece_count_for_player(opponent_player) as i32;

    let possible_moves: Vec<Move> = position.enumerate_moves().collect();
    if let GamePhase::Moving = game_phase {
        if own_stones < 3 { return -1_000_000; }
        if opponent_stones < 3 { return 1_000_000; }
        if possible_moves.is_empty() {
            return -1_000_000;
        }
    }

    position_value += (own_stones - opponent_stones) * 100;

    for line in MILL_LINES.iter() {
        let mut own_stones = 0;
        let mut opponent_stones = 0;
        let mut empty_field = 0;
        for &point in line.iter() {
            match position.get_piece_at_position(point) {
                Point::Occupied(player) => if player == active_player {
                    own_stones += 1
                } else { opponent_stones += 1 },
                _ => empty_field += 1,
            }
        }
        if own_stones == 2 && empty_field == 1 {
            position_value += 500;
        }
        if own_stones == 3{
            position_value += 1500;
        }
        if opponent_stones == 2 && empty_field == 1 {
            position_value -= 6500;
        }
        if opponent_stones == 3{
            position_value -= 900;
        }
    }

    for point in position.get_pieces_for_player(active_player) {
        if position.is_corner(point) {
            position_value += 5;
        } else {
            position_value += 40;
        }
        match game_phase {
            GamePhase::Moving => {
                if own_stones <= 3 {
                    break;
                }

                // bigger than 3
                let mut is_trapped = true;
                position.for_each_neighbour(point, |neighbor| {
                    if let Point::Occupied(player) = neighbor {
                        if player.ne(&active_player) && player.ne(&opponent_player) {
                            is_trapped = false
                        }
                    }
                });

                if is_trapped {
                    let is_my_mill = position.is_mill_closed(None, point, active_player);
                    let is_blocking = check_is_blocking(position, point);
                    if !is_my_mill && !is_blocking {
                        position_value -= 300;
                    }
                }

            }
            _ => {}
        }

    }

    // TODO
    let mut position1 = position.clone();
    position1.switch_player();
    let x1: Vec<Move> = position1.enumerate_moves().collect();
    position_value += ((possible_moves.len() as i32) - (x1.len() as i32)) * 20;
    position_value
}

fn check_is_blocking(position: &Position, point: u8) -> bool {
    let opponent_player = position.opposite_player();

    for line in MILL_LINES.iter() {
        if line.contains(&point) {
            let mut opponent_stones = 0;
            let mut own_stones = 0;

            for &line_point in line.iter() {
                let piece_at_line_point = position.get_piece_at_position(line_point);
                if piece_at_line_point.is_occupied_by_player(opponent_player) {
                    opponent_stones += 1;
                } else if line_point == point {
                    own_stones += 1;
                }
            }
            if opponent_stones == 2 && own_stones == 1 {
                return true;
            }
        }
    }
    false
}

fn parse_input() -> (GamePhase, Position) {
    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let mut input = input.trim().split(" ");
    let mut pieces_remaining: Option<u8> = None;

    let game_phase = input.next().expect("Failed to parse game phase");
    let phase = match game_phase {
        "P" => {
            let pieces_left_str = input.next().expect("Failed to parse pieces left");
            let pieces_left = pieces_left_str.parse().expect("Failed to parse string to u8");
            pieces_remaining = Some(pieces_left);

            GamePhase::Placing
        },
        "M" => GamePhase::Moving,
        _ => panic!("Unknown phase")
    };

    let pos_str = input.next().expect("Failed to parse position string");
    let mut position: Position = pos_str.parse().expect("Failed to parse position from string");
    if let Some(pieces) = pieces_remaining {
        position.set_pieces_to_place_for_player(position.active_player(), pieces);
    }

    (phase, position)
}