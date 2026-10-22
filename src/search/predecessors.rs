use crate::ADJACENT;
use crate::datastructures::point::Point;
use crate::datastructures::position::Position;

pub fn calc_predecessors(position: &Position) -> Vec<Position> {
    let mut predecessors: Vec<Position> = Vec::new();
    let predecessor_player = position.opposite_player();

    let current_piece_count = position.get_piece_count_for_player(position.active_player());

    for predecessor_index in 0..24u8 {
        if !position.get_piece_at_position(predecessor_index).is_occupied_by_player(predecessor_player) {
            continue
        };

        let piece_count = position.get_piece_count_for_player(predecessor_player);
        let possible_points: Vec<u8> = if piece_count == 3 {
            (0..24).collect()
        } else {
            ADJACENT[predecessor_index as usize].to_vec()
        };

        for possible_point in possible_points {
            if position.get_piece_at_position(possible_point).is_occupied() {
                continue
            }

            let mut position_clone = position.clone();
            position_clone.set_active_player(predecessor_player);
            position_clone.set_piece_at_position(possible_point, Point::Occupied(predecessor_player));
            position_clone.set_piece_at_position(predecessor_index, Point::Empty);

            if position.is_mill_closed(None, predecessor_index, predecessor_player) {

                if current_piece_count >= 9 {
                    continue
                }

                for possible_kill_position in 0..24u8 {
                    // wenn position leer und nicht der aktuelle
                    if !position_clone.get_piece_at_position(possible_kill_position).is_occupied() &&
                        possible_kill_position != predecessor_index {
                        let mut position_clone_clone = position_clone.clone();

                        position_clone_clone.set_piece_at_position(possible_kill_position, Point::Occupied(position.active_player()));
                        if position_clone_clone.is_piece_removal_possible(None, possible_kill_position) {
                            predecessors.push(position_clone_clone);
                        }
                    }
                }
            } else {
                predecessors.push(position_clone);
            }
        }
    }

    predecessors
}
