use std::fs::{File};
use std::io::{BufRead, BufReader, Error};
use std::str::FromStr;
use muehle_game::Position;

const INPUT_PATH: &str = "data/input_ex3.txt";
const OUTPUT_PATH: &str = "data/input_ex3.txt";

fn main() -> Result<(), Error>{

    // lese file
    let input = File::open(INPUT_PATH)?;
    let buffered_in = BufReader::new(input);

    let output = File::open(OUTPUT_PATH)?;
    let buffered_out = BufReader::new(output);

    for line in buffered_in.lines() {
        match line {
            Ok(string) => {
                let result = Position::from_str(string.as_str());
                match result {
                    Ok(position) => {
                        let x = position.enumerate_moves();


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