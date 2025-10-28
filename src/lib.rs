mod test;

const PIECE_COUNT: usize = 24;

#[derive(Copy, Clone, Eq, PartialEq)]
#[derive(Debug)]
pub enum Piece {
    WHITE,
    BLACK
}

#[derive(Clone)]
pub enum Player {
    WHITE,
    BLACK
}

impl Piece {
    pub fn from(player: &Player) -> Self {
        match player {
            Player::WHITE => Piece::WHITE,
            Player::BLACK => Piece::BLACK
        }
    }
}

#[derive(Clone)]
pub struct Position {
    pieces: [Option<Piece>; PIECE_COUNT],
    active_player: Player
}

impl Position {
    pub fn new() -> Self {
        Position {
            pieces: [None; PIECE_COUNT],
            active_player: Player::WHITE
        }
    }

    pub fn switch_player(&mut self) {
        self.active_player = match self.active_player {
            Player::WHITE => Player::BLACK,
            Player::BLACK => Player::WHITE
        };
    }

    pub fn get_stone_at_position(&self, index: u8) -> Option<Piece> {
        if !Self::is_index_valid(index) {
            return None;
        }

        self.pieces[index as usize]
    }

    pub fn is_move_valid(&self, move_action: &Move) -> bool {
        match move_action.action {
            MoveAction::PLACE {position} => {
                if !Self::is_index_valid(position) {
                    return false;
                }
                if self.get_stone_at_position(position).is_some() {
                    return false;
                }
                true
            },
            MoveAction::MOVE {from_position, to_position} |
            MoveAction::FLY {from_position, to_position} => {
                if !Self::is_index_valid(from_position) && !Self::is_index_valid(to_position) {
                    return false;
                }
                if self.get_stone_at_position(to_position).is_some() {
                    return false;
                }

                let option = self.get_stone_at_position(from_position);
                match option {
                    Some(piece) if piece == Piece::from(&self.active_player) => false,
                    _ => false
                };
                true
            },
        }
    }

    pub fn change_piece_position(&mut self, move_action: &Move) {
        if !self.is_move_valid(move_action) {
            return;
        }
        match move_action.action {
            MoveAction::PLACE {position} => {
                self.pieces[position as usize] = Some(Piece::from(&self.active_player));
            }
            MoveAction::MOVE {from_position, to_position} |
            MoveAction::FLY {from_position, to_position} => {
                if let Some(piece) = self.get_stone_at_position(from_position).take() {
                    self.pieces[to_position as usize] = Some(piece);
                }
            }
        }
    }

    pub fn is_index_valid(index: u8) -> bool {
        (index as usize) < PIECE_COUNT
    }

    pub fn pieces(&self) -> [Option<Piece>; 24] {
        self.pieces
    }

    pub fn active_player(&self) -> &Player {
        &self.active_player
    }
}

pub enum MoveAction {
    PLACE { position: u8 },
    MOVE { from_position: u8, to_position: u8 },
    FLY { from_position: u8, to_position: u8 }
}


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

