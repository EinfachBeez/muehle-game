use log::warn;
use std::fmt::{Display, Formatter, Write};
use std::io::Error;
use std::str::{FromStr};

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

const ADJACENT: [&[u8]; PIECE_COUNT] = [
    &[1, 7, 8],
    &[0, 2],
    &[1, 3, 10],
    &[2, 4],
    &[3, 5, 12],
    &[4, 6],
    &[5, 7, 14],
    &[6, 0],
    &[0, 9, 15],
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Player {
    White,
    Black
}

impl Player {
    fn opposite(&self) -> Player {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White
        }
    }
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

#[derive(Clone, Copy, Debug)]
pub enum GamePhase {
    Placing,
    Moving,
    Flying
}

#[derive(Clone, Copy, Debug)]
pub enum Move {
    PLACE { position: u8, kill: Option<u8> },
    MOVE { from_position: u8, to_position: u8, kill: Option<u8> }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::PLACE { position, kill } => {
                let mut format = format!("P {}", position);
                if kill.is_some() {
                    format.push_str(" T ");
                    format.push_str(kill.unwrap().to_string().as_str());
                }
                write!(f, "{}", format)
            }
            Move::MOVE { from_position, to_position, kill } => {
                let mut format = format!("M {} {}", from_position, to_position);
                if kill.is_some() {
                    format.push_str(" T ");
                    format.push_str(kill.unwrap().to_string().as_str());
                }
                write!(f, "{}", format)
            }
        }

    }
}

#[derive(Clone)]
pub struct Position {
    pieces: [Point; PIECE_COUNT],
    active_player: Player,
    pieces_to_place: (u8, u8)
}

impl Position {
    pub fn new(player: Player) -> Self {
        Position {
            pieces: [Point::Empty; PIECE_COUNT],
            active_player: player,
            pieces_to_place: (9, 9)
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

    pub fn is_move_valid(&self, game_phase: GamePhase, move_action: Move) -> bool {
        // TODO: Check if kill is allowed
        match game_phase {
            GamePhase::Placing => {
                match move_action {
                    Move::PLACE { position, kill } => {
                        if !Self::is_index_valid(position) {
                            return false;
                        }
                        if self.get_piece_at_position(position).is_occupied() {
                            return false;
                        }
                        true
                    }
                    _ => false
                }
            },
            GamePhase::Moving => {
                match move_action {
                    Move::MOVE { from_position, to_position, kill } => {
                        if !Self::is_index_valid(from_position) && !Self::is_index_valid(to_position) {
                            return false;
                        }
                        if let Point::Occupied(player) = self.get_piece_at_position(from_position) {
                            if player != self.active_player {
                                return false;
                            }
                        }
                        if !Self::is_move_adjacent(from_position, to_position) {
                            return false;
                        }

                        true
                    }
                    _ => false
                }
            },
            GamePhase::Flying => {
                match move_action {
                    Move::MOVE { from_position, to_position, kill } => {
                        if !Self::is_index_valid(from_position) && !Self::is_index_valid(to_position) {
                            return false;
                        }
                        if let Point::Occupied(player) = self.get_piece_at_position(from_position) {
                            if player != self.active_player {
                                return false;
                            }
                        }
                        if self.get_piece_at_position(to_position).is_occupied() {
                            return false;
                        }

                        true
                    }
                    _ => false
                }
            }
        }
    }

    pub fn change_piece_position(&mut self, game_phase: GamePhase, move_action: Move) {
        if !self.is_move_valid(game_phase, move_action) {
            warn!("Invalid move {move_action:?}");
            return;
        }
        match move_action {
            Move::PLACE {position, kill} => {
                self.set_piece_at_position(position, Point::Occupied(self.active_player));

                if kill.is_some() {
                    self.set_piece_at_position(kill.unwrap(), Point::Empty);
                }
            }
            Move::MOVE {from_position, to_position, kill} => {
                let point = self.get_piece_at_position(from_position);
                self.set_piece_at_position(from_position, Point::Empty);
                self.set_piece_at_position(to_position, point);

                if kill.is_some() {
                    self.set_piece_at_position(kill.unwrap(), Point::Empty);
                }
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

    pub fn list_possible_pieces_to_remove(&self, player: Player) -> Vec<u8> {
        self.pieces.iter().enumerate()
            .filter(move |&(i, &p)| {
                p == Point::Occupied(player) && !self.is_mill_closed(i as u8, player)
            })
            .map(|(i, _)| i as u8)
            .collect()
    }

    pub fn is_piece_removal_possible(&self, piece: u8) -> bool {
        let point = self.get_piece_at_position(piece);
        match point {
            Point::Occupied(player) if player != self.active_player => {
                // find stones outside from mills
                let no_mill_stones = self.list_possible_pieces_to_remove(player);
                no_mill_stones.is_empty() || no_mill_stones.contains(&piece)
            }
            _ => {
                warn!("No piece to collect");
                false
            }
        }
    }

    pub fn is_kill_possible(&self, piece: u8) -> bool {
        self.is_mill_closed(piece, self.active_player.opposite())
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

    fn get_game_phase_for_player(&self, player: Player) -> GamePhase {
        match player {
            Player::White => {
                if self.pieces_to_place.0 != 0 {
                    return GamePhase::Placing
                }
                let pieces_on_board = self.get_pieces_on_board_for_player(Player::White);
                if pieces_on_board <= 3 {
                    return GamePhase::Flying;
                }
            }
            Player::Black => {
                if self.pieces_to_place.1 != 0 {
                    return GamePhase::Placing
                }
                let pieces_on_board = self.get_pieces_on_board_for_player(Player::Black);
                if pieces_on_board <= 3 {
                    return GamePhase::Flying
                }
            }
        }

        GamePhase::Moving
    }

    pub fn enumerate_moves(&self) -> impl Iterator<Item = Move> {
        let player = self.active_player;
        let phase = self.get_game_phase_for_player(self.active_player);

        let _ = match phase {
            GamePhase::Placing => {
                let placement_iter = (0..PIECE_COUNT)
                    .filter(move |&x| !self.get_piece_at_position(x as u8).is_occupied());

                placement_iter.flat_map(move |pos| {
                    if self.is_mill_closed(pos as u8, player) {
                        (0..PIECE_COUNT)
                            .filter(move |&x| self.is_kill_possible(pos as u8))
                            .map(move |kill| {
                                Move::PLACE { position: pos as u8, kill: Some(kill as u8) }
                            })
                    } else {
                        Move::PLACE { position: pos as u8, kill: None }
                    }
                })
            }
            GamePhase::Moving => {}
            GamePhase::Flying => {}
        };




        // get all positions from active player
        /*match phase {
            GamePhase::Moving => {
                pieces_iter()
                    .filter(|(_, &p)| p == Point::Occupied(player))
                    .flat_map(move |(from, _)| {
                        ADJACENT[from].iter()
                            .copied()
                            .filter(move |&to| self.get_piece_at_position(to) == Point::Empty)
                            .flat_map(move |to| {
                                //if self.is_mill_closed(to as u8, player) {
                                //    no_mill_stones_from_enemy.iter().copied()
                                //        .map(move |k| Move::MOVE { from_position: from as u8, to_position: to as u8, kill: Some(k) })
                                //        .chain(once(Move::MOVE { from_position: from as u8, to_position: to as u8, kill: None }))
                                //} //else {
                                    //once(Move::MOVE { from_position: from as u8, to_position: to as u8, kill: None })
                                //}
                            })
                    })
            },

            GamePhase::Flying => {
                pieces_iter()
                    .filter(|(_, &p)| p == Point::Occupied(player))
                    .flat_map(move |(from, _)| {
                        empty_points.iter().copied().flat_map(move |to| {
                            /*if self.is_mill_closed(to as u8, player) {
                                no_mill_stones_from_enemy.iter().copied()
                                    .map(move |k| Move::FLY { from_position: from as u8, to_position: to as u8, kill: Some(k) })
                                    .chain(once(Move::FLY { from_position: from as u8, to_position: to as u8, kill: None }))
                            } else {
                                //once(Move::FLY { from_position: from as u8, to_position: to as u8, kill: None })
                            }*/
                        })
                    })
            },

            GamePhase::Placing => {
                pieces_iter()
                    .filter(|(_, &p)| p == Point::Empty)
                    .flat_map(move |(pos, _)| {
                        /*if self.is_mill_closed(pos as u8, player) {
                            no_mill_stones_from_enemy.iter().copied()
                                .map(move |k| Move::PLACE { position: pos as u8, kill: Some(k) })
                                .chain(once(Move::PLACE { position: pos as u8, kill: None }))
                        } else {
                            //once(Move::PLACE { position: pos as u8, kill: None })
                        }*/
                    })
            }
        }*/
        std::iter::empty()

    }

    fn get_pieces_on_board_for_player(&self, player: Player) -> u8 {
        let mut count = 0;
        for i in 0..self.pieces.len() {
            if let Point::Occupied(p) = self.get_piece_at_position(i as u8) {
                if player == p {
                    count += 1;
                }
            }
        }

        count
    }

    fn is_index_valid(index: u8) -> bool {
        (index as usize) < PIECE_COUNT
    }

    pub fn pieces(&self) -> [Point; PIECE_COUNT] {
        self.pieces
    }

    pub fn set_active_player(&mut self, active_player: Player) {
        self.active_player = active_player;
    }

    pub fn active_player(&self) -> Player {
        self.active_player
    }

    pub fn set_pieces_to_place(&mut self, pieces_to_place: (u8, u8)) {
        self.pieces_to_place = pieces_to_place;
    }

    pub fn pieces_to_place(&self) -> (u8, u8) {
        self.pieces_to_place
    }
}

impl FromStr for Position {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            panic!("String is empty");
        }

        let mut chars = s.chars();
        let player_char = chars.next().unwrap();

        let player = match player_char {
            'W' => Player::White,
            'B' => Player::Black,
            _ => panic!("Invalid format")
        };

        match chars.next() {
            Some('-') => (),
            _ => panic!("Invalid format")
        }

        let x: String = chars.collect();
        if x.len() != 24 {
            panic!("Invalid format");
        }

        let mut position = Position::new(player, );
        for (index, char) in x.chars().enumerate() {
            match char {
                'E' => {position.set_piece_at_position(index as u8, Point::Empty); ()},
                'W' =>  {position.set_piece_at_position(index as u8, Point::Occupied(Player::White)); ()},
                'B' => {position.set_piece_at_position(index as u8, Point::Occupied(Player::Black)); ()},
                _ => panic!("Invalid format")
            }
        }
        position.set_active_player(player);

        Ok(position)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut position_string = String::new();
        let active_player = self.active_player();
        position_string.push_str(if active_player == Player::White {"W"} else {"B"});

        let mut active_positions: Vec<u8> = Vec::new();
        for (i, &piece) in self.pieces().iter().enumerate() {
            match piece {
                Point::Occupied(player) => {
                    if active_player == player {
                        active_positions.push(i as u8);
                    }
                    position_string.push_str(
                        if player == Player::White { "W" } else { "B" }
                    )
                },
                _ => position_string.push_str("E")
            };
        }
        f.write_str(position_string.as_str())?;
        f.write_char('\n')?;

        for x in self.enumerate_moves() {

        }

        todo!();
        Ok(())
    }
}

pub fn calculate_next_position(position: &mut Position, move_action: Move) -> Option<Position> {
    if !position.is_move_valid(move_action) {
        return None;
    }

    let mut position_clone = position.clone();
    position_clone.change_piece_position(move_action);
    position_clone.switch_player();

    Some(position_clone)
}
