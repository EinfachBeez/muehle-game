use log::warn;

const MILLS: [[u8;3]; 16] = [
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

const PIECE_COUNT: usize = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Player {
    White,
    Black
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Point {
    Empty,
    Occupied(Player)
}

impl Point {
    fn is_occupied(&self) -> bool {
        match self {
            Point::Occupied(_) => true,
            _ => false
        }
    }
}

pub enum GamePhase {
    Placing {
        white_pieces_left: u8,
        black_pieces_left: u8
    },
    Moving {
        white_pieces_left: u8,
        black_pieces_left: u8
    },
    Flying {
        white_pieces_left: u8,
        black_pieces_left: u8
    }
}

#[derive(Clone)]
pub struct Position {
    pieces: [Point; PIECE_COUNT],
    active_player: Player
}

impl Position {
    pub fn new() -> Self {
        Position {
            pieces: [Point::Empty; PIECE_COUNT],
            active_player: Player::White
        }
    }

    pub fn switch_player(&mut self) {
        self.active_player = match self.active_player {
            Player::White => Player::Black,
            Player::Black => Player::White
        };
    }

    pub fn get_piece_at_position(&self, index: u8) -> Point {
        if !Self::is_index_valid(index) {
            panic!("Index out of bound")
        }

        self.pieces[index as usize]
    }

    pub fn set_piece_at_position(&mut self, index: u8, point: Point) -> Point {
        if !Self::is_index_valid(index) {
            panic!("Index out of bound")
        }

        let old_piece = self.pieces[index as usize];
        self.pieces[index as usize] = point;
        old_piece
    }

    pub fn is_move_valid(&self, move_action: &Move) -> bool {
        match move_action.action {
            MoveAction::PLACE {position} => {
                if !Self::is_index_valid(position) {
                    return false;
                }
                if self.get_piece_at_position(position).is_occupied() {
                    return false;
                }
                true
            },
            MoveAction::MOVE {from_position, to_position} => {
                if !Self::is_index_valid(from_position) && !Self::is_index_valid(to_position) {
                    return false;
                }
                let option = self.get_piece_at_position(from_position);
                match option {
                    Point::Occupied(player) if player != self.active_player => false,
                    _ => false
                };
                if !Self::is_move_adjacent(from_position, to_position) {
                    return false;
                }

                true
            },
            MoveAction::FLY {from_position, to_position} => {
                if !Self::is_index_valid(from_position) && !Self::is_index_valid(to_position) {
                    return false;
                }
                let option = self.get_piece_at_position(from_position);
                match option {
                    Point::Occupied(player) if player != self.active_player => false,
                    _ => false
                };
                if self.get_piece_at_position(to_position).is_occupied() {
                    return false;
                }

                true
            },
        }
    }

    pub fn change_piece_position(&mut self, move_action: &Move) {
        if !self.is_move_valid(move_action) {
            warn!("Invalid move {move_action:?}");
            return;
        }
        match move_action.action {
            MoveAction::PLACE {position} => {
                self.set_piece_at_position(position, Point::Occupied(self.active_player));
            }
            MoveAction::MOVE {from_position, to_position} |
            MoveAction::FLY {from_position, to_position} => {
                let point = self.get_piece_at_position(from_position);
                self.set_piece_at_position(from_position, Point::Empty);
                self.set_piece_at_position(to_position, point);


            }
        }
    }

    pub fn is_move_adjacent(from_position: u8, to_position: u8) -> bool {
        if !Self::is_index_valid(from_position) && !Self::is_index_valid(to_position) {
            return false;
        }
        if from_position == to_position {
            return false;
        }
        let from_mod_position = from_position % 8;
        let to_mod_position = to_position % 8;
        if from_mod_position == to_mod_position {
            return true;
        }
        // Nach links
        from_mod_position == 0 && 8 - to_mod_position - 1 == 0
            || from_mod_position != 0 && from_mod_position - 1 == to_mod_position
            // Nach rechts
            || to_mod_position == 0 && 8 - from_mod_position - 1 == 0
            || to_mod_position != 0 && to_mod_position - 1 == from_mod_position
    }

    pub fn choose_piece_to_remove(&mut self, piece: u8) {
        if !self.is_piece_removal_possible(piece) {
            warn!("Piece removal not possible");
            return;
        }
        self.set_piece_at_position(piece, Point::Empty);
    }

    pub fn is_piece_removal_possible(&self, piece: u8) -> bool {
        let point = self.get_piece_at_position(piece);
        match point {
            Point::Occupied(player) if player != self.active_player => {
                // find stones outside from mills
                let no_mill_stones: Vec<u8> = self.pieces.iter().enumerate()
                    .filter(|&(i, &p)| p == Point::Occupied(player) && !self.is_mill_closed(i as u8, player))
                    .map(|(i, _)| i as u8)
                    .collect();

                if no_mill_stones.contains(&piece) {
                    return true;
                }

                false
            }
            _ => {
                warn!("No piece to collect");
                false
            }
        }
    }

    fn is_mill_closed(&self, new_pos: u8, player: Player) -> bool {
        for mill in MILLS {
            if mill.contains(&new_pos)
                && mill.iter().all(|&i| self.get_piece_at_position(i) == Point::Occupied(player)) {
                return true
            }
        }
        false
    }

    fn does_mill_exists_for_player(&self, player: Player) -> bool {
        for i in 0..self.pieces.len() {
            return match self.pieces[i] {
                Point::Occupied(p) if player == p && self.is_mill_closed(i as u8, player) => true,
                _ => continue
            }
        }
        false
    }

    fn is_index_valid(index: u8) -> bool {
        (index as usize) < PIECE_COUNT
    }

    pub fn pieces(&self) -> [Point; PIECE_COUNT] {
        self.pieces
    }

    pub fn active_player(&self) -> Player {
        self.active_player
    }
}

#[derive(Debug)]
pub enum MoveAction {
    PLACE { position: u8 },
    MOVE { from_position: u8, to_position: u8 },
    FLY { from_position: u8, to_position: u8 }
}

#[derive(Debug)]
pub struct Move {
    action: MoveAction
}

impl Move {
    pub fn new(move_action: MoveAction) -> Self {
        Move {
            action: move_action
        }
    }
}

pub fn calculate_next_position(position: &mut Position, move_action: &Move) -> Option<Position> {
    if !position.is_move_valid(move_action) {
        return None;
    }

    let mut position_clone = position.clone();
    position_clone.change_piece_position(move_action);
    position_clone.switch_player();

    Some(position_clone)
}

