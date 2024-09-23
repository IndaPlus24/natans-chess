/// Used to track piece alignment and who's turn it is.
#[derive(PartialEq, Clone, Copy)]
enum Color {
    White,
    Black,
}

mod piece_mod;
use piece_mod::Piece;

struct Game {
    board: [Option<Piece>; 8 * 8],
    turn_owner: Color,
    turn_count: u32,
}

impl Game {
    pub fn new() -> Game {
        let template = [
            'R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R', // White side
            'p', 'p', 'p', 'p', 'p', 'p', 'p', 'p', //
            '0', '0', '0', '0', '0', '0', '0', '0', //
            '0', '0', '0', '0', '0', '0', '0', '0', //
            '0', '0', '0', '0', '0', '0', '0', '0', //
            '0', '0', '0', '0', '0', '0', '0', '0', //
            'p', 'p', 'p', 'p', 'p', 'p', 'p', 'p', //
            'R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R', // Black side
        ];

        // Default             Black Empty White
        // piece map is      0xFFFF00000000FFFF
        let white_map = 0x000000000000FFFF as u64;

        // The default board should not crash
        let board = Game::make_board(template, white_map).ok().unwrap();
        Game {
            board: board,
            turn_owner: Color::White, // White starts
            turn_count: 1,            // 1st turn
        }
    }
    pub fn make_board(template: [char; 64], white_map: u64) -> Result<[Option<Piece>; 64], String> {
        let mut board: [Option<Piece>; 64];
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

        let mut w_crucial = false;
        let mut b_crucial = false;

        for i in 0..64 {
            let rank = template[i];
            
            // Leave empty spots
            if rank == '0' {
                continue;
            }

            // Default to black
            let color = if (white_map >> i) & 1 == 1 { Color::White } else { Color:: Black };
            let piece = Piece::new(color, rank);

            // Track if either side got a crucial piece (a "King")
            if piece.is_crucial {
                match color {
                    Color::White => b_crucial = true,
                    Color::Black => b_crucial = true,
                }
            }

            board[i] = Some(piece);
        }

        if !w_crucial && !b_crucial {
            return Err("Both sides need at least one crucial piece".to_owned());
        }

        Ok(board)
    }

    // (0,0) is bottom left. (7,7) is top right.
    pub fn piece_at(&self, col: u8, row: u8) -> &Option<Piece> {
        &self.board[(row * 8 + col) as usize]
    }

    pub fn print_board(&self) {
        // Row 0 is the bottom, but the console draws top to bottom.
        for row in (0..8 as u8).rev() {
            println!("+---+---+---+---+---+---+---+---+");
            for col in 0..8 as u8 {
                if (col + row) & 1 == 1 {
                    print!("|\x1b[7m");
                }
                else {
                    print!("|");
                }

                if let Some(p) = self.piece_at(col, row) {
                    match p.color {
                        Color::White => print!("({})", p.rank),
                        Color::Black => print!("[{}]", p.rank)
                    };
                } 
                else {
                    print!("   ")
                }
                print!("\x1b[0m");
            }
            println!("|");
        }
        println!("+---+---+---+---+---+---+---+---+");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_and_print() {
        let g = Game::new();
        g.print_board();
    }
}