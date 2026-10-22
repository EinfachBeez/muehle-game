use crate::datastructures::player::Player;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Point {
    Empty,
    Occupied(Player)
}

impl Point {
    pub fn is_occupied(&self) -> bool {
        match self {
            Point::Occupied(_) => true,
            _ => false
        }
    }

    pub fn is_occupied_by_player(&self, player: Player) -> bool {
        match self {
            Point::Occupied(p) if p.eq(&player) => true,
            _ => false
        }
    }
}