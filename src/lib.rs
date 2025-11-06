use log::warn;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::once;
use std::str;
use std::str::{FromStr};
use either::Either;

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

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Player::White => write!(f, "White"),
            Player::Black => write!(f, "Black")
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

    fn is_occupied_by_player(&self, player: Player) -> bool {
        match self {
            Point::Occupied(p) if p.eq(&player) => true,
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

impl Display for GamePhase {
    fn fmt(&self, f: &mut Formatter) ->  std::fmt::Result  {
        match self {
            GamePhase::Placing => write!(f, "Placing"),
            GamePhase::Moving => write!(f, "Moving"),
            GamePhase::Flying => write!(f, "Flying"),
        }
    }
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
                let mut format = format!("P {}", position + 1);
                if let Some(kill_pos) = kill {
                    format.push_str(" T ");
                    let kill_value = kill_pos + 1;
                    format.push_str(kill_value.to_string().as_str());
                }
                write!(f, "{}", format)
            }
            Move::MOVE { from_position, to_position, kill } => {
                let mut format = format!("M {} {}", from_position + 1, to_position + 1);
                if let Some(kill_pos) = kill {
                    format.push_str(" T ");
                    let kill_value = kill_pos + 1;
                    format.push_str(kill_value.to_string().as_str());
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
                        if let Some(pos) = kill {
                            return self.is_piece_removal_possible(None, pos)
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
                        if let Some(pos) = kill {
                            return self.is_piece_removal_possible(Some(from_position), pos)
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
                        if let Some(pos) = kill {
                            return self.is_piece_removal_possible(Some(from_position), pos)
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

                if let Some(pos) = kill {
                    self.set_piece_at_position(pos, Point::Empty);
                }
            }
            Move::MOVE {from_position, to_position, kill} => {
                let point = self.get_piece_at_position(from_position);
                self.set_piece_at_position(from_position, Point::Empty);
                self.set_piece_at_position(to_position, point);

                if let Some(pos) = kill {
                    self.set_piece_at_position(pos, Point::Empty);
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

    pub fn is_piece_removal_possible(&self, old_piece: Option<u8>, piece: u8) -> bool {
        let point = self.get_piece_at_position(piece);
        match point {
            Point::Occupied(player) if player != self.active_player => {
                // does opposite player builds mill with piece
                let is_mill_closed = self.is_mill_closed(old_piece, piece, player);
                if is_mill_closed {
                    // check if the opposite player has only mills
                    for x in self.get_pieces_for_player(player) {
                        if x == piece {
                            continue;
                        }
                        if !self.is_mill_closed(old_piece, x, player) {
                            return false;
                        }
                    }
                    return true;
                }

                true
            }
            _ => {
                warn!("No piece to collect");
                false
            }
        }
    }

    pub fn is_mill_closed(&self, old_position: Option<u8>, new_position: u8, player: Player) -> bool {
        for mill in MILLS {
            if mill.contains(&new_position) {
                if mill.iter().all(|&i|
                    {
                        if i == new_position {
                            return true;
                        }
                        let mut is_old_position_in_array = false;
                        if let Some(pos) = old_position {
                            is_old_position_in_array = pos == i;
                        }
                        !is_old_position_in_array && self.get_piece_at_position(i) == Point::Occupied(player)
                    }
                ) {
                    return true
                }
            }
        }
        false
    }

    pub fn get_game_phase_for_current_player(&self) -> GamePhase {
        self.get_game_phase_for_player(self.active_player)
    }

    pub fn get_game_phase_for_player(&self, player: Player) -> GamePhase {
        match player {
            Player::White => {
                if self.pieces_to_place.0 != 0 {
                    return GamePhase::Placing
                }
                let pieces_on_board = self.get_piece_count_for_player(player);
                if pieces_on_board <= 3 {
                    return GamePhase::Flying;
                }
            }
            Player::Black => {
                if self.pieces_to_place.1 != 0 {
                    return GamePhase::Placing
                }
                let pieces_on_board = self.get_piece_count_for_player(player);
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

        match phase {
            GamePhase::Placing => {
                let placement_iter = (0..PIECE_COUNT)
                    .filter(move |&points| !self.get_piece_at_position(points as u8).is_occupied());

                Either::Left(placement_iter.flat_map(move |pos| {
                    let default_placement = Move::PLACE {position: pos as u8, kill: None};

                    if self.is_mill_closed(None, pos as u8, player) {
                        Either::Left(self.get_pieces_for_player(player.opposite())
                            .filter(move |&x| self.is_piece_removal_possible(None, x))
                            .map(move |kill| {
                                Move::PLACE { position: pos as u8, kill: Some(kill) }
                            }))
                    } else {
                       Either::Right(once(default_placement))
                    }
                }))
            }
            GamePhase::Moving => {
                let moving_iter = self.get_pieces_for_player(player);

                Either::Right(Either::Left(moving_iter.flat_map(move |pos| {
                    ADJACENT[pos as usize].iter()
                        .copied()
                        .filter(move |&adjacent| self.get_piece_at_position(adjacent) == Point::Empty)
                        .flat_map(move |to| {
                            let default_move = Move::MOVE {from_position: pos, to_position: to, kill: None};

                            if self.is_mill_closed(Some(pos), to, player) {
                                Either::Left(self.get_pieces_for_player(player.opposite())
                                    .filter(move |&x| self.is_piece_removal_possible(Some(pos), x))
                                    .map(move |kill| {
                                        Move::MOVE {from_position: pos, to_position: to, kill: Some(kill)}
                                    }))
                            } else {
                                Either::Right(once(default_move))
                            }
                        })
                })))
            }
            GamePhase::Flying => {
                let flying_iter = self.get_pieces_for_player(player);

                Either::Right(Either::Right(flying_iter.flat_map(move |pos| {
                    (0..PIECE_COUNT)
                        .filter(move |&point| self.get_piece_at_position(point as u8) == Point::Empty)
                        .flat_map(move |to| {
                            let default_fly = Move::MOVE {from_position: pos, to_position: to as u8, kill: None};

                            if self.is_mill_closed(Some(pos), to as u8, player) {
                                Either::Left(self.get_pieces_for_player(player.opposite())
                                    .filter(move |&x| self.is_piece_removal_possible(Some(pos), x))
                                    .map(move |kill| {
                                        Move::MOVE {from_position: pos, to_position: to as u8, kill: Some(kill)}
                                    }))
                            } else {
                                Either::Right(once(default_fly))
                            }
                        })
                })))
            }
        }

    }

    pub fn get_pieces_for_player(&self, player: Player) -> impl Iterator<Item = u8> {
        (0..PIECE_COUNT)
            .filter(move |&points| self.get_piece_at_position(points as u8).is_occupied_by_player(player))
            .map(move |x| x as u8)
    }

    pub fn get_piece_count_for_player(&self, player: Player) -> u8 {
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

    pub fn set_pieces_to_place_for_player(&mut self, player: Player, pieces_to_place: u8) {
        match player {
            Player::White => self.pieces_to_place.0 = pieces_to_place,
            Player::Black => self.pieces_to_place.1 = pieces_to_place,
        }

    }

    pub fn pieces_to_place_for_player(&self, player: Player) -> u8 {
        match player {
            Player::White => self.pieces_to_place.0,
            Player::Black => self.pieces_to_place.1
        }
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

        let mut position = Position::new(player);
        let (mut w, mut b) = (9, 9);
        for (index, char) in x.chars().enumerate() {
            match char {
                'E' => {position.set_piece_at_position(index as u8, Point::Empty); ()},
                'W' =>  {
                    position.set_piece_at_position(index as u8, Point::Occupied(Player::White));
                    w -= 1;
                    ()
                },
                'B' => {
                    b -= 1;
                    position.set_piece_at_position(index as u8, Point::Occupied(Player::Black));
                    ()
                },
                _ => panic!("Invalid format")
            }
        }
        position.set_pieces_to_place((w, b));
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
        Ok(())
    }
}

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
