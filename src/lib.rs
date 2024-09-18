/// Used to track piece alignment and who's turn it is.
enum Color {
    White,
    Black,
}

mod piece_mod;
use piece_mod::Piece;

struct Game {
    board: [Option<Piece>; 8 * 8],
    turn: Color,
}



impl Game {
    fn new() -> Game {
        let board: [Option<Piece>; 8 * 8];
        board = [
            None, None, None, None, None, None, None, None, // a
            None, None, None, None, None, None, None, None, // a
            None, None, None, None, None, None, None, None, // a
            None, None, None, None, None, None, None, None, // a
            None, None, None, None, None, None, None, None, // a
            None, None, None, None, None, None, None, None, // a
            None, None, None, None, None, None, None, None, // a
            None, None, None, None, None, None, None, None, // a
        ];
        Game {
            board: board,
            turn: Color::White,
        }
    }

    pub fn piece_at(&self, col: u8, row: u8) -> Option<Piece> {
        None
    }
}
