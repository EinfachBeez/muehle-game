use std::fmt::{Display, Formatter};
use std::str::FromStr;
use anyhow::anyhow;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Move {
    Place { position: u8, kill: Option<u8> },
    Move { from_position: u8, to_position: u8, kill: Option<u8> }
}

impl FromStr for Move {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut string = s.split_whitespace();

        match string.next().unwrap() {
            "P" => {
                let position: u8 = string.next().unwrap().parse()?;

                let kill = match string.next() {
                    Some("K") => {
                        let kill_value = string.next().unwrap();
                        Some(kill_value.parse()?)
                    },
                    _ => None
                };

                Ok(Move::Place { position, kill })
            }
            "M" => {
                let from_position: u8 = string.next().unwrap().parse()?;
                let to_position: u8 = string.next().unwrap().parse()?;

                let kill = match string.next() {
                    Some("K") => {
                        let kill_value = string.next().unwrap();
                        Some(kill_value.parse()?)
                    },
                    _ => None
                };

                Ok(Move::Move { from_position, to_position, kill })
            }
            _ => {
                Err(anyhow!("Unknown action"))
            }
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Place { position, kill } => {
                let mut format = format!("P {}", position + 1);
                if let Some(kill_pos) = kill {
                    format.push_str(" T ");
                    let kill_value = kill_pos + 1;
                    format.push_str(kill_value.to_string().as_str());
                }
                write!(f, "{}", format)
            }
            Move::Move { from_position, to_position, kill } => {
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