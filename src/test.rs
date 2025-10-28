#[cfg(test)]
mod tests {
    use muehle_game::{calculate_next_position, Move, MoveAction, Piece, Position};

    #[test]
    fn test_set_and_get() {
        let mut pos = Position::new();
        let m = Move::new(MoveAction::PLACE {position: 3});
        pos.change_piece_position(&m);
        assert_eq!(pos.get_stone_at_position(3), Some(Piece::WHITE));
        print_board(&pos)
    }

    #[test]
    fn test_move() {
        let mut pos = Position::new();
        let m = Move::new(MoveAction::PLACE {position: 23});
        let mut pos1 = calculate_next_position(&mut pos, &m).unwrap();
        print_board(&pos1);
        let m = Move::new(MoveAction::MOVE {from_position: 23, to_position: 3});
        let pos2 = calculate_next_position(&mut pos1, &m).unwrap();
        print_board(&pos2);
        assert_eq!(pos2.get_stone_at_position(3), Some(Piece::WHITE));
    }


    fn print_board(position: &Position) {
        let x = position.pieces();
        let char_val = |i| match x[i] {
            Some(Piece::WHITE) => 'W',
            Some(Piece::BLACK) => 'B',
            _ => '.',
        };

        println!("
 {}---------{}---------{}
 |         |         |
 |   {}-----{}-----{}   |
 |   |     |     |   |
 |   |  {}--{}--{}  |   |
 {}--{}--{}     {}--{}--{}
 |   |  {}--{}--{}  |   |
 |   |     |     |   |
 |   {}-----{}-----{}   |
 |         |         |
 {}---------{}---------{}",
                 char_val(0), char_val(1), char_val(2),
                 char_val(3), char_val(4), char_val(5),
                 char_val(6), char_val(7), char_val(8),
                 char_val(9), char_val(10), char_val(11), char_val(12), char_val(13), char_val(14),
                 char_val(15), char_val(16), char_val(17),
                 char_val(18), char_val(19), char_val(20),
                 char_val(21), char_val(22), char_val(23)
        );

    }

}