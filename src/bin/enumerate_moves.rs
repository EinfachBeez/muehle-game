use std::fs::{File};
use std::io::{BufRead, BufReader, Error};
use std::str::FromStr;
use muehle_game::{Position};

const INPUT_PATH: &str = "src/data/input_ex3.txt";
const OUTPUT_PATH: &str = "src/data/input_ex3.txt";

fn main() -> Result<(), Error>{

    // lese file
    let input = File::open(INPUT_PATH)?;
    let buffered_in = BufReader::new(input);

    let output = File::open(OUTPUT_PATH)?;
    let mut buffered_out = BufReader::new(output).lines();

    for line in buffered_in.lines() {
        let line_content = buffered_out.next().unwrap().expect("Failed to read line");
        match line {
            Ok(input_position) => {
                if !input_position.eq(&line_content) {
                   panic!("Err")
                }
                println!("{input_position}");
                let result = Position::from_str(input_position.as_str());
                match result {
                    Ok(mut position) => {
                        position.set_pieces_to_place((0, 0));
                        let x = position.enumerate_moves();

                        for move_action in x {
                            println!("{move_action}");
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