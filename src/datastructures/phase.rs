use std::fmt::{Display, Formatter};

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