pub mod datastructures;

use crate::datastructures::movement::Move;
use crate::datastructures::phase::GamePhase;
use crate::datastructures::position::Position;

pub const MILLS: [[u8;3]; 16] = [
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
    [6, 14, 22]

];

const ADJACENT: [&[u8]; PIECE_COUNT] = [
    &[1, 7, 8],
    &[0, 2],
    &[1, 3, 10],
    &[2, 4],
    &[3, 5, 12],
    &[4, 6],
    &[5, 7, 14],
    &[6, 0],
    &[0, 9, 15, 16],
    &[8, 10],
    &[2, 9, 11, 18],
    &[10, 12],
    &[4, 11, 13, 20],
    &[12, 14],
    &[6, 13, 15, 22],
    &[8, 14],
    &[8, 17, 23],
    &[16, 18],
    &[10, 17, 19],
    &[18, 20],
    &[12, 19, 21],
    &[20, 22],
    &[14, 21, 23],
    &[16, 22]
];

const PIECE_COUNT: usize = 24;

pub fn calculate_next_position(position: &Position, move_action: Move) -> Option<Position> {
    let active_player = position.active_player();
    let phase = position.get_game_phase_for_player(active_player);
    if !position.is_move_valid(phase, move_action) {
        return None;
    }

    let mut position_clone = position.clone();
    position_clone.change_piece_position(phase, move_action);
    match phase {
        GamePhase::Placing => {
            let i = position_clone.pieces_to_place_for_player(active_player);
            position_clone.set_pieces_to_place_for_player(active_player, i-1);
        }
        _ => ()
    }
    position_clone.switch_player();

    Some(position_clone)
}