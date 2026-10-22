use std::cmp::min;
use crate::datastructures::position::Position;

pub fn calc_canonical_representative(position: &Position) -> Position {
    let mut canonical_position = position.to_bits();

    let og_position = position.to_bits();
    let swapped_pos = swap_colors_and_player(og_position);

    for pos in [og_position, swapped_pos] {
    for i in 0..4 {
        let pos_rotate = rotate_position(pos, i);
        for j in 0..2 {
            let pos_mirror = mirror_position_horizontal(pos_rotate, j);
            for k in 0..2 {
                let pos_flip = flip_position(pos_mirror, k);
                canonical_position = min(canonical_position, pos_flip);
            }
        }
    }
    }

    Position::from_bits(canonical_position)
}

fn swap_colors_and_player(mut pos: u64) -> u64 {
    // 1. Aktiven Spieler flippen (Bit 63 umschalten)
    pos ^= 1u64 << 63;

    // 2. Steine auf dem Brett tauschen (Nur die unteren 48 Bits!)
    // Maske für ungerade Bits (..010101) in den unteren 48 Bits
    let mask_01: u64 = 0x5555_5555_5555_5555;
    // Maske für gerade Bits (..101010) in den unteren 48 Bits
    let mask_10: u64 = 0xAAAA_AAAA_AAAA_AAAA;

    // Wir isolieren den Brett-Teil (untere 48 Bits)
    // Alles darüber (außer Bit 63, das wir schon geflippt haben) lassen wir in Ruhe.
    let board_mask: u64 = 0x0000_FFFF_FFFF_FFFF;

    let board = pos & board_mask;
    let meta = pos & !board_mask; // Bit 63 und eventuelle Zähler

    // Der eigentliche Swap-Trick:
    // Nimm alle 01er, schieb sie nach links (werden 10)
    let ones_to_twos = (board & mask_01) << 1;
    // Nimm alle 10er, schieb sie nach rechts (werden 01)
    let twos_to_ones = (board & mask_10) >> 1;

    // Alles wieder zusammensetzen
    (ones_to_twos | twos_to_ones) | meta
}

fn rotate_position(position: u64, times: usize) -> u64 {
    let times = times % 4;
    let mut pos: u64 = position;
    for ring in 0..3 {
        let ring_slice = position_ring_splice(pos, ring);
        let result = ring_slice.rotate_right(4 * times as u32);
        pos &= !(0xFFFF << (ring * 16));
        pos |= (result as u64) << (ring * 16);
    }

    pos
}

fn mirror_position_horizontal(position: u64, times: usize) -> u64 {
    mirror_position(position, times, 0)
}


fn mirror_position(position: u64, times: usize, vertical: usize) -> u64 {
    let times = times % 2;
    let vertical = vertical % 2;
    let start_pos = if vertical == 0 { 1 } else { 3 };

    let mut pos: u64 = position;
    for _ in 0..times {
        for ring in 0..3 {
            let mut ring_slice = position_ring_splice(position, ring);
            for i in 0..3 {
                // 6,4,2 or 0,7,6
                let shift = (6 + start_pos - i) % 8;

                let left_position_shift = (2 * (i + start_pos)) % 16;
                let right_position_shift = (shift * 2) % 16;

                // 6,4,2,2,4,6
                let swap_1_val = ring_slice >> left_position_shift & 0b11;
                let swap_2_val = ring_slice >> right_position_shift & 0b11;

                // clear and swap values
                ring_slice &= !(0b11 << left_position_shift | 0b11 << right_position_shift);

                ring_slice |= swap_1_val << right_position_shift;
                ring_slice |= swap_2_val << left_position_shift;
            }

            pos &= !(0xFFFF << (ring * 16));
            pos |= (ring_slice as u64) << (ring * 16);
        }
    }


    pos
}

fn flip_position(position: u64, times: usize) -> u64 {
    let times = times % 2;
    let mut result = position;
    for _ in 0..times {
        let ring_0_slice = position_ring_splice(position, 0);
        let ring_2_slice = position_ring_splice(position, 2);

        //0xFFFF0000FFFF
        let mask: u64 = 0xFFFF_u64 | (0xFFFF_u64 << 32);

        let clear = result & !mask;
        result = clear | ((ring_0_slice as u64) << 32) | ring_2_slice as u64
    }
    result
}

fn position_ring_splice(position: u64, ring: u8) -> u16 {
    (position >> (ring * 16) & 0xFFFF) as u16
}
