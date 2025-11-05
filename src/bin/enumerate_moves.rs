use std::fs::{write, File};
use std::io::{BufRead, BufReader, BufWriter, Error, Write};
use std::process::exit;
use std::str::FromStr;
use muehle_game::{Move, Position};

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Syntax: cargo run --bin enumerate_moves -- <Input_file.txt> <Output_file.txt>");
        exit(1);
    }

    let input = &args[1];
    let output = &args[2];

    let input = File::open(input)?;
    let output = File::create(output)?;
    let buffered_in = BufReader::new(input);
    let mut buffered_out = BufWriter::new(output);

    for line in buffered_in.lines() {
        match line {
            Ok(input_position) => {
                let result = Position::from_str(input_position.as_str());
                match result {
                    Ok(mut position) => {
                        position.set_pieces_to_place((0, 0));
                        let x = position.enumerate_moves();

                        let mut collected_moves: Vec<Move> = x.collect();
                        collected_moves.sort_by(move |x1, x2| x1.to_string().cmp(&x2.to_string()));

                        writeln!(buffered_out, "{}", input_position)?;

                        for move_action in collected_moves {
                            writeln!(buffered_out, "{}", move_action)?;
                        }

                    }
                    _ => panic!("Err")
                }
            }
            _ => panic!("Err")
        }
    }

    buffered_out.flush()?;
    println!("Finished!");
    Ok(())

}