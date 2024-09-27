use core::fmt::Display;

/// Used to track piece alignment and who's turn it is.
#[derive(PartialEq, Clone, Copy, Debug)]
enum Color {
    White,
    Black,
}

impl Display for Color {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::White => write!(f, "White"),
            Color::Black => write!(f, "Black"),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum GameState {
    Running,
    Check,
    GameOver,
    /// Hopefully I will never have to use this one.
    /// But I would rather have it and not need it, than need it and not have it.
    SomethingHasGoneTerriblyWrongMilord,
}

mod piece_mod;
use std::collections::HashSet;

use piece_mod::*;

#[derive(Clone)]
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

    pub fn make_move(&mut self, from: (u8, u8), to: (u8, u8)) -> bool {
        println!("Moving from ({},{})", from.0, from.1);
        if let Some(piece) = self.piece_at(from.0, from.1) {
            // Do not move the opponent's piece
            if self.turn_owner != piece.color {
                println!("{} can not move {}'s pieces", self.turn_owner, piece.color);
                return false;
            }

            let moves = piece.all_possible_moves(from.0, from.1, self);

            // Does it have the move?????
            if let Some(effects) = moves.get(&(to.0 + to.1 * 8)) {
                if !self.is_safe_move(from, to, effects, piece.color) {
                    print!(
                        "The move {} ({},{}) to ({},{}) is not safe",
                        piece.rank, from.0, from.1, to.0, to.1
                    );
                    return false;
                }
                /// IT DO! AND IT SAFE!
                self.just_execute_move(from, to, effects);

                match self.turn_owner {
                    Color::White => {
                        self.turn_owner = Color::Black;
                    }
                    Color::Black => {
                        self.turn_owner = Color::White;
                        self.turn_count += 1;
                    }
                }
                return true;
            }
            print!("If you are seeing this, then things have gone terribly wrong.");
        }
        println!("No piece found at ({},{})", from.0, from.1);
        false
    }

    /// This will perform the move without checking if ANYTHING is legal.
    /// Caution is advised when calling directly
    fn just_execute_move(&mut self, from: (u8, u8), to: (u8, u8), effects: &Vec<Effect>) {
        self.just_move(from, to);
        for e in effects {
            match e {
                Effect::Capture(p) => self.capture(position(*p, from)),
                Effect::Move(p1, p2) => self.just_move(position(*p1, from), position(*p2, from)),
            }
        }
    }

    /// This will force pieces to move. Will crash if there is no piece to move.
    fn just_move(&mut self, from: (u8, u8), to: (u8, u8)) {
        let piece = self.board[(from.0 + from.1 * 8) as usize]
            .clone()
            .expect("DO NOT USE just_move IF YOU DO NOT KNOW WHAT YOU ARE DOING!");

        let piece2 = Piece {
            last_moved: Some(self.turn_count),
            times_moved: piece.times_moved + 1,
            ..piece
        };

        self.board[(to.0 + to.1 * 8) as usize] = Some(piece2);
        self.board[(from.0 + from.1 * 8) as usize] = None;
    }

    /// Will remove the piece, no questions asked.
    fn capture(&mut self, pos: (u8, u8)) {
        self.board[(pos.0 + pos.1 * 8) as usize] = None;
    }

    // (0,0) is bottom left. (7,7) is top right.
    pub fn piece_at(&self, col: u8, row: u8) -> Option<&Piece> {
        if col > 7 || row > 7 {
            return None;
        }
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
                        Color::Black => print!("<{}>", p.rank),
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

    /// The color refers to who it is safe FOR, not from.
    pub fn is_safe_position(&self, col: u8, row: u8, color: Color) -> bool {
        // Assume that every move that they can make in response is safe.
        // This should (emphasis on should) stop any and all infinite loops.
        if color != self.turn_owner {
            return true;
        }

        for i in 0..64 as u8 {
            if let Some(piece) = self.piece_at(i % 8, i >> 3) {
                if piece.color == color {
                    continue;
                }
                let a = piece.get_danger_zone(i % 8, i >> 3, self);
                if a.contains(&(col + row * 8)) {
                    return false;
                }
            }
        }
        true
    }

    fn is_safe_move(
        &self,
        from: (u8, u8),
        to: (u8, u8),
        effects: &Vec<Effect>,
        color: Color,
    ) -> bool {
        let mut gc = self.clone();
        gc.just_execute_move(from, to, effects);

        let mut i = 0;
        for p in &gc.board {
            if let Some(piece) = p {
                if piece.is_crucial
                    && piece.color == color
                    && !gc.is_safe_position(i % 8, i >> 3, color)
                {
                    return false;
                }
            }
            i += 1;
        }
        return true;
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
    fn danger_zone() {
        std::env::set_var("RUST_BACKTRACE", "1");
        let g = Game::new();
        let mut i = 0 as u8;

        for o in &g.board {
            if let Some(p) = o {
                println!("Testing {} at {},{}", p.rank, i % 8, i >> 3);
                p.get_danger_zone(i % 8, i >> 3, &g);
            }
            i += 1;
        }
    }

    #[test]
    fn test_pawn_move_normal() {
        let from = (4 as u8, 1 as u8);
        let to = (4 as u8, 2 as u8);

        let mut g = Game::new();
        let p = g.piece_at(from.0, from.1).unwrap();
        let m = p.moves[0].prune(&g, from);

        if m.len() != 1 || !m.contains_key(&(to.0 + to.1 * 8)) {
            panic!();
        }

        println!("Move success: {}", g.make_move(from, to));
        let a = g.piece_at(to.0, to.1);

        println!("Debug Piece: {:#?}", a);
        g.print_board();

        if let Some(p) = a {
            if p.color == Color::White && p.last_moved == Some(1) && p.times_moved == 1 {
                return;
            }
        }
        panic!();
    }

    #[test]
    fn test_pawn_move_double() {
        let from = (4 as u8, 1 as u8);
        let to = (4 as u8, 3 as u8);
        let to2 = (4 as u8, 5 as u8);

        let mut g = Game::new();
        let p = g.piece_at(from.0, from.1).unwrap();
        let m = p.moves[1].prune(&g, from);

        if m.len() != 1 || !m.contains_key(&(to.0 + to.1 * 8)) {
            panic!();
        }

        println!("Move success: {}", g.make_move(from, to));
        let a = g.piece_at(to.0, to.1);

        println!("Debug Piece: {:#?}", a);
        g.print_board();

        if let Some(p) = a {
            if p.color != Color::White || p.last_moved != Some(1) || p.times_moved != 1 {
                panic!();
            }
        }
        else {
            panic!();
        }

        // Also make sure it can not move in such a way twice
        println!("Move success (should be false): {}", g.make_move(from, to));

        let b = g.piece_at(to2.0, to2.1);
    
        if let Some(p) = b {
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
        let start = (3 as u8, 3 as u8);
        let goal = (4 as u8, 4 as u8);
        
        template[(start.0 + start.1 * 8) as usize] = 'p';
        template[(goal.0 + goal.1 * 8) as usize] = 'p';

        let b = Game::make_board(template, color_template).ok().unwrap();
        
        let mut g = Game {
            board: b,
            turn_owner: Color::White,
            turn_count: 1,
            game_state: GameState::Running,
        };

        g.print_moves(start.0, start.1);

        let p = g.piece_at(start.0, start.1).unwrap();
        let m = p.moves[2].prune(&g, start);

        if m.len() != 1 || !m.contains_key(&(goal.0 + goal.1 * 8)) {
            panic!();
        }

        println!("Move success: {}", g.make_move(start, goal));
        let a = g.piece_at(goal.0, goal.1);

        println!("Debug Piece: {:#?}", a);
        g.print_board();

        if let Some(p) = a {
            if p.color == Color::White && p.last_moved == Some(1) && p.times_moved == 1 {
                return;
            }
        }
        panic!();
    }

    #[test]
    fn test_pawn_move_en_passant() {
        let mut template = test_template;
        let start = (3 as u8, 3 as u8);
        let subgoal = (3 as u8, 4 as u8);
        let goal = (4 as u8, 5 as u8);
        let start2 = (4 as u8, 6 as u8);
        let goal2 = (4 as u8, 4 as u8);

        
        template[(start.0 + start.1 * 8) as usize] = 'p';
        template[(start2.0 + start2.1 * 8) as usize] = 'p';

        let b = Game::make_board(template, color_template).ok().unwrap();
        
        let mut g = Game {
            board: b,
            turn_owner: Color::White,
            turn_count: 1,
            game_state: GameState::Running,
        };

        println!("Move part 1 success: {}", g.make_move(start, subgoal));

        println!("Move part 2 success: {}", g.make_move(start2, goal2));
        
        println!("Move part 3 success: {}", g.make_move(subgoal, goal));

        let a = g.piece_at(goal.0, goal.1);
        let b = g.piece_at(goal2.0, goal2.1);

        println!("Debug Piece: {:#?}", a);
        g.print_board();

        if let Some(p) = a {
            if let Some(_p) = b {
                panic!()
            }
            if p.color == Color::White && p.last_moved == Some(2) && p.times_moved == 2 {
                return;
            }
        }
        panic!();
    }

    #[test]
    fn test_pawn_move_capture_true_flip() {
        let mut template = test_template;
        let start = (4 as u8, 3 as u8);
        let goal = (3 as u8, 4 as u8);
        
        template[(start.0 + start.1 * 8) as usize] = 'p';
        template[(goal.0 + goal.1 * 8) as usize] = 'p';

        let b = Game::make_board(template, color_template).ok().unwrap();
        
        let mut g = Game {
            board: b,
            turn_owner: Color::White,
            turn_count: 1,
            game_state: GameState::Running,
        };

        g.print_moves(start.0, start.1);

        let p = g.piece_at(start.0, start.1).unwrap();
        let m = p.moves[2].prune(&g, start);

        if m.len() != 1 || !m.contains_key(&(goal.0 + goal.1 * 8)) {
            panic!();
        }

        println!("Move success: {}", g.make_move(start, goal));
        let a = g.piece_at(goal.0, goal.1);

        println!("Debug Piece: {:#?}", a);
        g.print_board();

        if let Some(p) = a {
            if p.color == Color::White && p.last_moved == Some(1) && p.times_moved == 1 {
                return;
            }
        }
        panic!();
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
