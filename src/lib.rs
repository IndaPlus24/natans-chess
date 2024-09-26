/// Used to track piece alignment and who's turn it is.
#[derive(PartialEq, Clone, Copy)]
enum Color {
    White,
    Black,
}

#[derive(PartialEq)]
enum GameState {
    Running,
    Check,
    GameOver,
    /// Hopefully I will never have to use this one.
    /// But I would rather have it and not need it, than need it and not have it.
    SomethingHasGoneTerriblyWrongMilord,
}

mod piece_mod;
use piece_mod::Piece;

struct Game {
    board: [Option<Piece>; 8 * 8],
    turn_owner: Color,
    turn_count: u32,
    game_state: GameState,
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
            game_state: GameState::Running,
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
            let color = if (white_map >> i) & 1 == 1 {
                Color::White
            } else {
                Color::Black
            };
            let piece = Piece::new(color, rank);

            // Track if either side got a crucial piece (a "King")
            if piece.is_crucial {
                match color {
                    Color::White => w_crucial = true,
                    Color::Black => b_crucial = true,
                }
            }

            board[i] = Some(piece);
        }

        if !w_crucial || !b_crucial {
            return Err("Both sides need at least one crucial piece".to_owned());
        }

        Ok(board)
    }

    // (0,0) is bottom left. (7,7) is top right.
    pub fn piece_at(&self, col: u8, row: u8) -> Option<&Piece> {
        self.board[(row * 8 + col) as usize].as_ref()
    }

    pub fn print_board(&self) {
        // Row 0 is the bottom, but the console draws top to bottom.
        for row in (0..8 as u8).rev() {
            for col in 0..8 as u8 {
                if (col + row) & 1 == 1 {
                    print!("\x1b[7m");
                }

                if let Some(p) = self.piece_at(col, row) {
                    match p.color {
                        Color::White => print!("({})", p.rank),
                        Color::Black => print!("[{}]", p.rank),
                    };
                } else {
                    print!("   ")
                }
                print!("\x1b[0m");
            }
            println!();
        }
    }

    pub fn print_moves(&self, col: u8, row: u8) {
        if let Some(p) = self.piece_at(col, row) {
            let moves = p.all_possible_moves(col, row, self);

            for r in (0..8 as u8).rev() {
                for c in 0..8 as u8 {
                    if c == col && r == row {
                        print!("\x1b[38;5;9m\x1b[48;5;1m");
                    } else if moves.contains_key(&(c + r * 8)) {
                        print!("\x1b[38;5;14m\x1b[48;5;14m");
                    }
                    if (c + r) & 1 == 1 {
                        print!("\x1b[7m");
                    }

                    if let Some(p) = self.piece_at(c, r) {
                        match p.color {
                            Color::White => print!("({})", p.rank),
                            Color::Black => print!("[{}]", p.rank),
                        };
                    } else {
                        print!("   ")
                    }
                    print!("\x1b[0m");
                }
                // println!("|");
                println!();
            }
        }
    }

    // I promise, I will make it actually do things soon.
    pub fn is_safe_position(&self, col: u8, row: u8, color: Color) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const test_template: [char; 64] = [
        '0', 'K', '0', '0', '0', '0', '0', '0', //
        '0', '0', '0', '0', '0', '0', '0', '0', //
        '0', '0', '0', '0', '0', '0', '0', '0', //
        '0', '0', '0', '0', '0', '0', '0', '0', //
        '0', '0', '0', '0', '0', '0', '0', '0', //
        '0', '0', '0', '0', '0', '0', '0', '0', //
        '0', '0', '0', '0', '0', '0', '0', '0', //
        '0', '0', '0', '0', '0', '0', 'K', '0', //
    ];
    const color_template: u64 = 0x00000000FFFFFFFF;
    #[test]
    fn make_and_print() {
        let g = Game::new();
        g.print_board();
    }

    #[test]
    fn display_moves() {
        let g = Game::new();
        g.print_moves(4, 1);
        g.print_moves(5, 1);
    }

    #[test]
    fn test_pawn_move_normal() {
        let g = Game::new();
        let p = g.piece_at(4, 1).unwrap();
        let m = p.moves[0].prune(&g, (4, 1));

        if m.len() != 1 || !m.contains_key(&(4 + 2 * 8)) {
            panic!();
        }
    }

    #[test]
    fn test_pawn_move_double() {
        let g = Game::new();
        let p = g.piece_at(4, 1).unwrap();
        let m = p.moves[1].prune(&g, (4, 1));

        if m.len() != 1 || !m.contains_key(&(4 + 3 * 8)) {
            panic!();
        }
    }

    #[test]
    fn test_pawn_move_capture_false() {
        let g = Game::new();
        let p = g.piece_at(4, 1).unwrap();
        let m = p.moves[2].prune(&g, (4, 1));

        if m.len() != 0 {
            panic!();
        }
    }

    #[test]
    fn test_pawn_move_capture_true() {
        let mut template = test_template;
        template[3 + 3 * 8] = 'p';
        template[4 + 4 * 8] = 'p';
        let b = Game::make_board(template, color_template).ok().unwrap();
        let g = Game {
            board: b,
            turn_owner: Color::White,
            turn_count: 0,
            game_state: GameState::Running,
        };

        g.print_moves(3, 3);

        let p = g.piece_at(3, 3).unwrap();
        let m = p.moves[2].prune(&g, (3, 3));

        if m.len() != 1 || !m.contains_key(&(4 + 4 * 8)) {
            panic!();
        }
    }

    #[test]
    fn test_pawn_move_capture_true_flip() {
        let mut template = test_template;
        template[4 + 3 * 8] = 'p';
        template[3 + 4 * 8] = 'p';
        let b = Game::make_board(template, color_template).ok().unwrap();
        let g = Game {
            board: b,
            turn_owner: Color::White,
            turn_count: 0,
            game_state: GameState::Running,
        };

        g.print_moves(4, 3);

        let p = g.piece_at(4, 3).unwrap();
        let m = p.moves[2].prune(&g, (4, 3));

        if m.len() != 1 || !m.contains_key(&(3 + 4 * 8)) {
            panic!();
        }
    }

    #[test]
    fn test_pawn_black_move_normal() {
        let g = Game::new();
        let p = g.piece_at(4, 6).unwrap();
        let m = p.moves[0].prune(&g, (4, 6));

        g.print_moves(4, 6);

        if m.len() != 1 || !m.contains_key(&(4 + 5 * 8)) {
            panic!();
        }
    }

    #[test]
    fn test_pawn_black_move_double() {
        let g = Game::new();
        let p = g.piece_at(4, 6).unwrap();
        let m = p.moves[1].prune(&g, (4, 6));

        g.print_moves(4, 6);

        if m.len() != 1 || !m.contains_key(&(4 + 4 * 8)) {
            panic!();
        }
    }

    #[test]
    fn test_pawn_black_move_capture_false() {
        let g = Game::new();
        let p = g.piece_at(4, 6).unwrap();
        let m = p.moves[2].prune(&g, (4, 6));

        g.print_moves(4, 6);

        if m.len() != 0 {
            panic!();
        }
    }

    #[test]
    fn test_pawn_black_move_capture_true() {
        let mut template = test_template;
        template[3 + 3 * 8] = 'p';
        template[4 + 4 * 8] = 'p';
        let b = Game::make_board(template, color_template).ok().unwrap();
        let g = Game {
            board: b,
            turn_owner: Color::White,
            turn_count: 0,
            game_state: GameState::Running,
        };

        g.print_moves(4, 4);

        let p = g.piece_at(4, 4).unwrap();
        let m = p.moves[2].prune(&g, (4, 4));

        if m.len() != 1 || !m.contains_key(&(3 + 3 * 8)) {
            panic!();
        }
    }

    #[test]
    fn test_pawn_black_move_capture_true_flip() {
        let mut template = test_template;
        template[4 + 3 * 8] = 'p';
        template[3 + 4 * 8] = 'p';
        let b = Game::make_board(template, color_template).ok().unwrap();
        let g = Game {
            board: b,
            turn_owner: Color::White,
            turn_count: 0,
            game_state: GameState::Running,
        };

        g.print_moves(3, 4);

        let p = g.piece_at(3, 4).unwrap();
        let m = p.moves[2].prune(&g, (3, 4));

        if m.len() != 1 || !m.contains_key(&(4 + 3 * 8)) {
            panic!();
        }
    }
}
