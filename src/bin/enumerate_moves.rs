use std::fs::{File};
use std::io::{BufRead, BufReader, Error};
use std::str::FromStr;
use muehle_game::{Move, Position};

const INPUT_PATH: &str = "data/input_ex3.txt";
const OUTPUT_PATH: &str = "data/input_ex3.txt";

fn main() -> Result<(), Error>{

    // lese file
    let input = File::open(INPUT_PATH)?;
    let buffered_in = BufReader::new(input);

    let output = File::open(OUTPUT_PATH)?;
    let mut buffered_out = BufReader::new(output).lines();

    for line in buffered_in.lines() {
        let line_content = buffered_out.next().unwrap().expect("Failed to read line");
        match line {
            Ok(string) => {
                if string.eq(&line_content) {
                   println!("Correct")
                }
                let result = Position::from_str(string.as_str());
                match result {
                    Ok(position) => {
                        let x = position.enumerate_moves();

                        for move_action in x {
                            match move_action {
                                Move::MOVE {from_position, to_position, kill} |
                                Move::FLY {from_position, to_position, kill} => {
                                    let possible_move = buffered_out.next().unwrap().expect("TODO: panic message");
                                    let mut format = format!("M {from_position} {to_position} ");
                                    if kill.is_some() {
                                        format.push_str("T ");
                                        format.push_str(kill.unwrap().to_string().as_str());
                                    }

                                    if !possible_move.eq(&format) {
                                        println!("Uff");
                                    }
                                }
                                _ => continue
                            }
                        }
                    }
                    _ => panic!("Err")
                }
            }
            _ => panic!("Err")
        }
    }




    Ok(())

    // werfe das aus der hilfsdinger aus der lib rein

    //


}