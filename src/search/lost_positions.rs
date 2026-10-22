use std::collections::HashSet;
use crate::ADJACENT;
use crate::datastructures::player::Player;
use crate::datastructures::point::Point;
use crate::datastructures::position::Position;

pub fn calc_lost_positions_with_k<F>(mut callback: F, k: usize) where F: FnMut(Position) {
    calc_two_stones_lost(&mut callback, k);
    calc_blocked_lost(&mut callback, k);
}

fn empty_position_black_to_move() -> Position {
    let mut pos = Position::new(Player::Black);
    pos.set_pieces_to_place((0, 0));
    pos
}

pub fn calc_two_stones_lost<F>(callback: &mut F, k: usize) where F: FnMut(Position) {
    for i in 0..24 {
        for j in (i+1)..24 {
            let mut pos = empty_position_black_to_move();
            pos.set_piece_at_position(i as u8, Point::Occupied(Player::Black));
            pos.set_piece_at_position(j as u8, Point::Occupied(Player::Black));
            let empty_fields = get_empty_fields(i, j);
            for v in 3..=k {
                let mut current_white_positions = Vec::new();
                calc_white_positions(&empty_fields, v, 0, &mut current_white_positions, &pos, callback);
            }
        }
    }
}


fn get_empty_fields(first_black_stone: usize, second_black_stone: usize) -> Vec<usize> {
    let mut empty_fields = Vec::new();
    for i in 0..24 {
        if i != first_black_stone && i != second_black_stone {
            empty_fields.push(i);
        }
    }
    empty_fields
}

fn calc_white_positions<F>(
    empty_fields: &[usize],
    amount_white_stones: usize,
    start_index: usize,
    current_white_positions: &mut Vec<usize>,
    original_positions: &Position,
    callback: &mut F,
) where F: FnMut(Position) {
    if current_white_positions.len() == amount_white_stones {
        let mut pos_clone = original_positions.clone();
        for index in current_white_positions.iter(){
            pos_clone.set_piece_at_position(*index as u8, Point::Occupied(Player::White));
        }
        callback(pos_clone);
        return;
    }
    for i in start_index..empty_fields.len() {
        current_white_positions.push(empty_fields[i]);
        calc_white_positions(empty_fields, amount_white_stones, i+1, current_white_positions, original_positions, callback);
        current_white_positions.pop();
    }
}

fn calc_blocked_lost<F>(mut callback: F, k: usize) where F: FnMut(Position) {
    let all_indices: Vec<usize> = (0..24).collect();

    for black_count in 4..=k {

        calc_possible_positions(&all_indices, black_count, |black_indices| {
            let mut base_pos = empty_position_black_to_move();

            for &i in &black_indices {
                base_pos.set_piece_at_position(i as u8, Point::Occupied(Player::Black));
            }

            let mut neighbour_white: HashSet<usize> = HashSet::new();
            for &i in &black_indices {
                for &neighbor in ADJACENT[i] {
                    let index = neighbor as usize;
                    if !black_indices.contains(&index) {
                        neighbour_white.insert(index);
                    }
                }
            }

            if neighbour_white.len() > 9 {
                return;
            }
            for &i in &neighbour_white {
                base_pos.set_piece_at_position(i as u8, Point::Occupied(Player::White));
            }

            let mut free_fields = Vec::new();
            for i in 0..24 {
                if !base_pos.get_piece_at_position(i as u8).is_occupied() {
                    free_fields.push(i);
                }
            }

            let min_white = neighbour_white.len().max(4);

            for remaining_white in min_white..=k {
                let remaining = remaining_white - neighbour_white.len();
                // Nicht genug freie Felder
                if remaining > free_fields.len() {
                    continue;
                }

                calc_possible_positions(&free_fields, remaining, |extra_white| {
                    let mut pos = base_pos.clone();

                    for &i in &extra_white {
                        pos.set_piece_at_position(i as u8, Point::Occupied(Player::White));
                    }

                    if is_player_blocked(&pos, Player::Black) {
                        callback(pos);
                    }
                });
            }
        });
    }
}

fn is_player_blocked(position: &Position, player: Player) -> bool {
    position.get_pieces_for_player(player).all(|point| {
        ADJACENT[point as usize].iter().all(|&neighbor| position.get_piece_at_position(neighbor).is_occupied())
    })
}

fn calc_possible_positions<F>(items: &[usize], pieces: usize, mut callback: F) where F: FnMut(Vec<usize>) {
    fn recursive_func<F>(
        items: &[usize],
        k: usize,
        start: usize,
        current: &mut Vec<usize>,
        callback: &mut F,
    ) where
        F: FnMut(Vec<usize>),
    {
        if current.len() == k {
            callback(current.clone());
            return;
        }

        for i in start..items.len() {
            current.push(items[i]);
            recursive_func(items, k, i + 1, current, callback);
            current.pop();
        }
    }

    let mut current = Vec::new();
    recursive_func(items, pieces, 0, &mut current, &mut callback);
}
