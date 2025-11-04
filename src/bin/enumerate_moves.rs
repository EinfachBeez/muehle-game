use std::fs::{File};
use std::io::{BufRead, BufReader, Error};
use std::str::FromStr;
use muehle_game::{Move, Position};

const INPUT_PATH: &str = "src/data/input_ex3.txt";
const OUTPUT_PATH: &str = "src/data/output_ex3.txt";

fn main() -> Result<(), Error>{

    let input = File::open(INPUT_PATH)?;
    let buffered_in = BufReader::new(input);

    let output = File::open(OUTPUT_PATH)?;
    let mut buffered_out = BufReader::new(output).lines();

    for line in buffered_in.lines() {
        let line_content = buffered_out.next().unwrap().expect("Failed to read line");
        match line {
            Ok(input_position) => {
                if !input_position.eq(&line_content) {
                    println!("{} == {}", input_position, line_content);
                    println!("{}", buffered_out.next().unwrap().expect("Failed to read line"));
                    break;
                }
                println!("{input_position}");
                let result = Position::from_str(input_position.as_str());
                match result {
                    Ok(mut position) => {
                        position.set_pieces_to_place((0, 0));
                        let x = position.enumerate_moves();

                        let mut collected_moves: Vec<Move> = x.collect();
                        collected_moves.sort_by(move |x1, x2| x1.to_string().cmp(&x2.to_string()));

                        for move_action in collected_moves {
                            println!("{move_action}");

                            let string = buffered_out.next().unwrap().expect("Failed to read line");
                            if !move_action.to_string().eq(&string) {
                                println!("Error: Not an equal move. Is: {move_action}, Should: {string}");
                                break;
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

}