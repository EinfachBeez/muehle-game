#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use muehle_game::{calculate_next_position, GamePhase, Move, Player, Position};
    use muehle_game::Point::Occupied;

    #[test]
    fn test_set_and_get() {
        let mut pos = Position::new(Player::White);
        let m = Move::PLACE {position: 3, kill: None};
        pos.change_piece_position(GamePhase::Placing, m);
        assert_eq!(pos.get_piece_at_position(3), Occupied(Player::White));
        print_board(&pos)
    }

    #[test]
    fn test_move() {
        let mut pos = Position::new(Player::White);
        let m = Move::PLACE {position: 23, kill: None};
        let mut pos1 = calculate_next_position(&mut pos, GamePhase::Placing, m).unwrap();
        print_board(&pos1);
        let m = Move::MOVE {from_position: 23, to_position: 3, kill: None};
        let pos2 = calculate_next_position(&mut pos1, GamePhase::Placing, m).unwrap();
        print_board(&pos2);
        assert_eq!(pos2.get_piece_at_position(3), Occupied(Player::White));
    }

    #[test]
    fn test_adjacent() {
        let mut pos = Position::new(Player::White);
        let place_piece = Move::PLACE {position: 9, kill: None};
        pos.change_piece_position(GamePhase::Placing, place_piece);
        let move_piece = Move::MOVE {from_position: 9, to_position: 16, kill: None};
        let x = pos.is_move_valid(GamePhase::Placing,move_piece);
        assert_eq!(true, x);

        let move_piece = Move::MOVE {from_position: 8, to_position: 0, kill: None};
        let x = pos.is_move_valid(GamePhase::Placing,move_piece);
        assert_eq!(true, x);

        let move_piece = Move::MOVE {from_position: 0, to_position: 7, kill: None};
        let x = pos.is_move_valid(GamePhase::Placing,move_piece);
        assert_eq!(true, x);

        let move_piece = Move::MOVE {from_position: 7, to_position: 5, kill: None};
        let x = pos.is_move_valid(GamePhase::Placing,move_piece);
        assert_eq!(false, x);

        let move_piece = Move::MOVE {from_position: 7, to_position: 6, kill: None};
        let x = pos.is_move_valid(GamePhase::Placing,move_piece);
        assert_eq!(true, x);
    }

    #[test]
    fn test_input() {
        let input = "W-WWBEEEEWEWBBBEWWBWWBBEBE";
        let result = Position::from_str(input);
        match result {
            Ok(mut position) => {
                position.set_pieces_to_place((0, 0));
                let x1 = position.enumerate_moves();
                for x in x1 {
                    println!("{x}")
                }
            }
            _ => panic!("Err")
        }
    }


    fn print_board(position: &Position) {
        let x = position.pieces();
        let char_val = |i| match x[i] {
            Occupied(Player::White) => 'W',
            Occupied(Player::Black) => 'B',
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