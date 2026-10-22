use std::collections::VecDeque;
use hashbrown::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::exit;
use std::time::Instant;
use muehle_game::calculate_next_position;
use muehle_game::datastructures::position::Position;
use muehle_game::search::canonical::calc_canonical_representative;
use muehle_game::search::lost_positions::calc_lost_positions_with_k;
use muehle_game::search::predecessors::calc_predecessors;

use peak_alloc::PeakAlloc;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

// Bit 63 ist sonst der Spieler am Zug, der kanonische Vertreter hat aber immer Weiß am Zug.
const WIN_FLAG: u64 = 1 << 63;
const POS_MASK: u64 = WIN_FLAG - 1;
type CompactPos = [u8; 6];

fn pack(pos: &Position, is_win: bool) -> u64 {
    let val = pos.to_bits();
    if is_win { val | WIN_FLAG } else { val & POS_MASK }
}

fn unpack(val: u64) -> (Position, bool) {
    (Position::from_bits(val & POS_MASK), (val & WIN_FLAG) != 0)
}

fn compact_position(pos: &Position) -> CompactPos {
    let val = pos.to_bits();
    let byte_slice = val.to_le_bytes();
    [byte_slice[0], byte_slice[1], byte_slice[2], byte_slice[3], byte_slice[4], byte_slice[5]]
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Syntax: cargo run --bin complete_search -- Eingabedatei.txt Ausgabedatei.txt");
        exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let reader = BufReader::new(File::open(input_path)?);
    let mut output_file = File::create(output_path)?;

    let mut lines = reader.lines();
    let first_line = lines.next().expect("No lines")?;
    let k: usize = first_line.trim().parse()?;

    let sharding_value = (k + 1) * (k + 1);
    let mut shards: Vec<HashSet<u64>> = (0..sharding_value)
        .map(|_| HashSet::with_capacity(32))
        .collect();

    let mut queue: VecDeque<u64> = VecDeque::with_capacity(100_000);
    println!("Start computing k = {}...", k);
    let start = Instant::now();

    calc_lost_positions_with_k(|pos| {
        let canon = calc_canonical_representative(&pos);
        let shard = &mut shards[shard_for_position(&canon, k)];
        let packed = pack(&canon, false);
        if shard.insert(packed) {
            queue.push_back(packed);
        }
    }, k);

    compute_win_loss_positions(k, &mut shards, &mut queue);

    let static_shards: Vec<Box<[u64]>> = shards.into_iter()
        .map(|set| {
            let mut v: Vec<u64> = set.into_iter().collect();
            v.sort_unstable();
            v.into_boxed_slice()
    }).collect();

    process_input(k, lines, &static_shards, &mut output_file)?;

    println!("Finished in {:?}", start.elapsed());
    println!("Current RAM: {} MB", PEAK_ALLOC.current_usage_as_mb());
    println!("Peak RAM: {} GB", PEAK_ALLOC.peak_usage_as_gb());
    Ok(())
}

fn process_input(
    k: usize,
    lines: std::io::Lines<BufReader<File>>,
    shards: &[Box<[u64]>],
    out: &mut File
) -> Result<(), Box<dyn Error>> {
    for line in lines {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue
        }

        let pos: Position = match trimmed.parse() {
            Ok(p) => p,
            Err(_) => {
                continue
            },
        };
        let canon_pos = calc_canonical_representative(&pos);
        let shard = &shards[shard_for_position(&canon_pos, k)];

        let packed_lost_position = pack(&canon_pos, false);
        let packed_won_position = pack(&canon_pos, true);

        let result = if shard.binary_search(&packed_lost_position).is_ok() {
            0
        } else if shard.binary_search(&packed_won_position).is_ok() {
            2
        } else {
            1
        };

        writeln!(out, "{}", result)?;
    }
    Ok(())
}

pub fn compute_win_loss_positions(k: usize, shards: &mut Vec<HashSet<u64>>, queue: &mut VecDeque<u64>) {
    let mut open_successors: HashMap<CompactPos, u8> = HashMap::with_capacity(100_000);

    while let Some(packed_val) = queue.pop_front() {
        let (position, is_win) = unpack(packed_val);
        if is_win {
            mark_won(k, shards, queue, &position, &mut open_successors);
        } else {
            mark_lost(k, shards, queue, &position);
        }
    }
}

fn mark_lost(k: usize, shards: &mut Vec<HashSet<u64>>, queue: &mut VecDeque<u64>, position: &Position) {
    for pred in calc_predecessors(position) {
        if has_more_than_k_stones(&pred, k) { continue }
        let canon_pos = calc_canonical_representative(&pred);
        let shard = &mut shards[shard_for_position(&canon_pos, k)];

        if !shard.contains(&pack(&canon_pos, false)) && !shard.contains(&pack(&canon_pos, true)) {
            let new_win = pack(&canon_pos, true);
            shard.insert(new_win);
            queue.push_back(new_win);
        }
    }
}

fn mark_won(k: usize, shards: &mut Vec<HashSet<u64>>, queue: &mut VecDeque<u64>, position: &Position, open_successors: &mut HashMap<CompactPos, u8>) {
    let mut local_buf = Vec::with_capacity(16);

    for pred in calc_predecessors(position) {
        if has_more_than_k_stones(&pred, k) {
            continue
        }

        let canon_pos = calc_canonical_representative(&pred);
        let position_value = canon_pos.to_bits();
        if local_buf.contains(&position_value) {
            continue
        }
        local_buf.push(position_value);

        let shard = &mut shards[shard_for_position(&canon_pos, k)];

        if !shard.contains(&pack(&canon_pos, false)) && !shard.contains(&pack(&canon_pos, true)) {
            let compact = compact_position(&canon_pos);
            let count = open_successors.entry(compact).or_insert_with(|| count_unique_canonical_successors(&pred) as u8);

            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    let new_loss = pack(&canon_pos, false);
                    if shard.insert(new_loss) {
                        queue.push_back(new_loss);
                    }
                    open_successors.remove(&compact);
                }
            }
        }
    }
}

fn shard_for_position(pos: &Position, k: usize) -> usize {
    let active_count = pos.get_piece_count_for_player(pos.active_player()) as usize;
    let opponent_count = pos.get_piece_count_for_player(pos.opposite_player()) as usize;
    active_count * (k + 1) + opponent_count
}

fn count_unique_canonical_successors(pos: &Position) -> usize {
    let mut s = Vec::with_capacity(20);
    for mv in pos.enumerate_moves() {
        s.push(calc_canonical_representative(&calculate_next_position(pos, mv).unwrap()).to_bits());
    }
    s.sort_unstable(); s.dedup();
    s.len()
}

fn has_more_than_k_stones(pos: &Position, k: usize) -> bool {
    pos.get_piece_count_for_player(pos.active_player()) as usize > k
        || pos.get_piece_count_for_player(pos.opposite_player()) as usize > k
}
