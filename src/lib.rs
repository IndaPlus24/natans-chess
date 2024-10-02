use core::fmt::Display;

/// Used to track piece alignment and who's turn it is.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Color {
    White,
    Black,
}

/// Moves such as Castling and en passant affect pieces on squares other than the one they land on.\
/// These are used to describe such effects.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    /// Capture the piece at the position.
    Capture(Position),
    /// Moves the piece from the first to the second.
    Move(Position, Position),
}

/// Position stuff
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Position {
    /// ALWAYS relative to the where "owner" of the move is when they start the move.
    Relative((i8, i8)),
    Global((u8, u8)),
}

impl Position {
    /// Adds up the sum of two positions as you would expect them to.\
    /// - A global position + a global position returns None (it breaks)\
    /// - A global position + a relative position returns a global position\
    /// - A relative position + a relative position returns a relative position
    pub fn add(self, p2: Position) -> Option<Position> {
        match self {
            Position::Global((x, y)) => match p2 {
                Position::Global(_f) => None,
                Position::Relative((dx, dy)) => Some(Position::Global((
                    (x as i8 + dx) as u8,
                    (y as i8 + dy) as u8,
                ))),
            },
            Position::Relative((dx, dy)) => match p2 {
                Position::Global((x, y)) => Some(Position::Global((
                    (x as i8 + dx) as u8,
                    (y as i8 + dy) as u8,
                ))),
                Position::Relative((dx2, dy2)) => Some(Position::Relative((dx + dx2, dy + dy2))),
            },
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Color::White => write!(f, "White"),
            Color::Black => write!(f, "Black"),
        }
    }
}

/// Describes, you guessed it, the state of the game.
#[derive(PartialEq, Clone, Copy)]
pub enum GameState {
    /// Everything is running fine.
    Running,
    /// You need to promote pieces before you can continue.
    Promote,
    /// The turn owner is currently in check.
    Check,
    /// The turn owner is currently in check.
    CheckMate,
    /// It is a stalemate, and the game is a draw.
    Stalemate,
    /// Hopefully I will never have to use this one.
    /// But I would rather have it and not need it, than need it and not have it.
    SomethingHasGoneTerriblyWrongMilord,
}

mod piece_mod;
use std::collections::*;

use piece_mod::*;

/// The thing with all the things in it!
#[derive(Clone)]
pub struct Game {
    board: [Option<Piece>; 8 * 8],
    turn_owner: Color,
    turn_count: u32,
    game_state: GameState,
}

impl Game {
    /// Make a new, completely normal board.
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

    /// In case you want to set up a custom board.
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

    /// Will move the piece, if it deems the move to be legal, of course.
    pub fn make_move(&mut self, from: (u8, u8), to: (u8, u8)) -> bool {
        match self.game_state {
            GameState::Running | GameState::Check => {}
            _ => return false,
        }

        // println!("Moving from ({},{})", from.0, from.1);
        if let Some(piece) = self.get_piece_at(from.0, from.1) {
            // Do not move the opponent's piece
            if self.turn_owner != piece.color {
                // println!("{} can not move {}'s pieces", self.turn_owner, piece.color);
                return false;
            }

            let moves = piece.get_all_possible_moves(from.0, from.1, self);

            // Does it have the move?????
            if let Some(effects) = moves.get(&(to.0 + to.1 * 8)) {
                self.just_execute_move(from, to, effects);
                // IT DO!!!!!!!!!

                // Do not move on until every single piece is promoted.
                if self.game_state != GameState::Promote {
                    self.increment_turn();
                }

                return true;
            }
            // print!("If you are seeing this, then things have gone terribly wrong.");
        }
        // println!("No piece found at ({},{})", from.0, from.1);
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

    /// This will force pieces to move. Will crash if there is no piece to move because I can not be bothered to make it check first.
    fn just_move(&mut self, from: (u8, u8), to: (u8, u8)) {
        let piece = self.board[(from.0 + from.1 * 8) as usize]
            .clone()
            .expect("DO NOT USE just_move IF YOU DO NOT KNOW WHAT YOU ARE DOING!");

        let piece2 = Piece {
            last_moved: Some(self.turn_count),
            times_moved: piece.times_moved + 1,
            ..piece
        };

        if piece2.can_promote {
            if to.1
                == match piece2.color {
                    Color::White => 7,
                    Color::Black => 0,
                }
            {
                // Promotion time
                self.game_state = GameState::Promote;
            }
        }

        self.board[(to.0 + to.1 * 8) as usize] = Some(piece2);
        self.board[(from.0 + from.1 * 8) as usize] = None;
    }

    /// Will remove the piece, no questions asked.
    fn capture(&mut self, pos: (u8, u8)) {
        self.board[(pos.0 + pos.1 * 8) as usize] = None;
    }

    /// (0,0) is bottom left. (7,7) is top right.\
    /// Due to the structure of this thing, the default doc file does not
    /// include any information about the Piece struct. However, you should
    /// be able to find all the necessary stuff in the README file.
    pub fn get_piece_at(&self, col: u8, row: u8) -> Option<&Piece> {
        if col > 7 || row > 7 {
            return None;
        }
        self.board[(row * 8 + col) as usize].as_ref()
    }

    /// Why would you use this? Why did I make this public?
    pub fn print_board(&self) {
        // Row 0 is the bottom, but the console draws top to bottom.
        for row in (0..8 as u8).rev() {
            for col in 0..8 as u8 {
                if (col + row) & 1 == 1 {
                    print!("\x1b[7m");
                }

                if let Some(p) = self.get_piece_at(col, row) {
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

    /// Why would you use this? Why did I make this public?
    pub fn print_moves(&self, col: u8, row: u8) {
        if let Some(p) = self.get_piece_at(col, row) {
            let moves = p.get_all_possible_moves(col, row, self);

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

                    if let Some(p) = self.get_piece_at(c, r) {
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

    /// The color you give as an argument refers to who the space is safe FOR, not from.
    pub fn is_safe_position(&self, col: u8, row: u8, color: Color) -> bool {
        // Assume that every move that they can make in response is safe.
        // This should (emphasis on should) stop any and all infinite loops.
        if color != self.turn_owner {
            return true;
        }

        for i in 0..64 as u8 {
            if let Some(piece) = self.get_piece_at(i % 8, i >> 3) {
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

    /// The color you give as an argument refers to who the space is safe FOR, not from.
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

    /// Get (a copy of) the piece that needs to be promoted.
    /// If you stick to the default pieces, then there should be no issue.
    pub fn get_promotion(&self) -> Option<((u8, u8), Piece)> {
        if self.game_state != GameState::Promote {
            return None;
        }
        println!("Looking for a piece to promote.");
        let row: u8 = 7;
        for col in 0..8 as u8 {
            if let Some(p) = self.get_piece_at(col, row) {
                if p.can_promote && p.color == Color::White {
                    return Some(((col, row), p.clone()));
                }
            }
        }

        let row: u8 = 0;
        for col in 0..8 as u8 {
            if let Some(p) = self.get_piece_at(col, row) {
                if p.can_promote && p.color == Color::Black {
                    return Some(((col, row), p.clone()));
                }
            }
        }
        None
    }

    /// Returns false when the promotion failed.\
    /// Will change the state when you are done.\
    /// You can try promoting pieces not returned by get_promotion, but it will probably fail.
    pub fn promote(&mut self, pos: (u8, u8), rank: char) -> bool {
        if self.game_state != GameState::Promote {
            return false;
        }

        if let Some(p) = self.get_piece_at(pos.0, pos.1) {
            if !p.can_promote || p.rank == rank {
                return false;
            }

            if (!(p.color == Color::White && pos.1 == 7)
                && (!(p.color == Color::Black && pos.1 == 0)))
            {
                return false;
            }

            let template_piece = Piece::new(p.color, rank);

            if template_piece.is_crucial || template_piece.can_promote {
                return false;
            }

            let promoted_piece = Piece {
                last_moved: p.last_moved,
                times_moved: p.times_moved,
                ..template_piece
            };

            self.board[(pos.0 + pos.1 * 8) as usize] = Some(promoted_piece);

            if self.get_promotion().is_none() {
                self.game_state = GameState::Running;

                self.increment_turn();
            }

            true
        } else {
            false
        }
    }

    /// Check what the state of the game is.
    pub fn get_game_state(&self) -> GameState {
        self.game_state
    }

    /// Figure out who's turn it is.
    pub fn get_turn_owner(&self) -> Color {
        self.turn_owner
    }

    /// Gets a hashmap of all possible moves a piece at the specified
    /// position can make with, together with the move's Effect.\
    /// Since the positions are the "keys", I recommend that
    /// you iterate across the whole thing.\
    /// By the way, if you want to actually get a position from the
    /// singe u8 this thing gives you, do the following
    /// ```
    /// let pos = // Imagine that it somehow gets a position from the hashmap.
    /// let col = pos % 8;
    /// let row = pose >> 3;
    /// ```
    /// Quick side note; the engine caches no results, ever. So, consider caching it yourself.
    pub fn get_moves(&self, col: u8, row: u8) -> Option<HashMap<u8, Vec<Effect>>> {
        if let Some(p) = self.get_piece_at(col, row) {
            return Some(p.get_all_possible_moves(col, row, self));
        }
        None
    }

    fn has_moves(&self) -> bool {
        for i in 0..64 {
            if let Some(p) = self.get_piece_at(i % 8, i >> 3) {
                if p.color != self.turn_owner {
                    continue;
                }
            } else {
                continue;
            }

            if let Some(m) = self.get_moves(i % 8, i >> 3) {
                if m.len() > 0 {
                    return true;
                }
            }
        }
        false
    }

    fn in_check(&self) -> bool {
        for i in 0..64 {
            if let Some(p) = self.get_piece_at(i % 8, i >> 3) {
                if p.is_crucial
                    && p.color == self.turn_owner
                    && !self.is_safe_position(i % 8, i >> 3, p.color)
                {
                    return true;
                }
            }
        }
        false
    }

    fn increment_turn(&mut self) {
        match self.turn_owner {
            Color::White => {
                self.turn_owner = Color::Black;
            }
            Color::Black => {
                self.turn_owner = Color::White;
                self.turn_count += 1;
            }
        }

        if self.in_check() {
            self.game_state = GameState::Check;
        } else {
            self.game_state = GameState::Running;
        }

        if !self.has_moves() {
            self.game_state = match self.game_state {
                GameState::Check => GameState::CheckMate,
                _ => GameState::Stalemate,
            }
        }
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
        let p = g.get_piece_at(from.0, from.1).unwrap();
        let m = p.moves[0].prune(&g, from);

        if m.len() != 1 || !m.contains_key(&(to.0 + to.1 * 8)) {
            panic!();
        }

        println!("Move success: {}", g.make_move(from, to));
        let a = g.get_piece_at(to.0, to.1);

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
        let p = g.get_piece_at(from.0, from.1).unwrap();
        let m = p.moves[1].prune(&g, from);

        if m.len() != 1 || !m.contains_key(&(to.0 + to.1 * 8)) {
            panic!();
        }

        println!("Move success: {}", g.make_move(from, to));
        let a = g.get_piece_at(to.0, to.1);

        println!("Debug Piece: {:#?}", a);
        g.print_board();

        if let Some(p) = a {
            if p.color != Color::White || p.last_moved != Some(1) || p.times_moved != 1 {
                panic!();
            }
        } else {
            panic!();
        }

        // Also make sure it can not move in such a way twice
        println!("Move success (should be false): {}", g.make_move(from, to));

        let b = g.get_piece_at(to2.0, to2.1);

        if let Some(p) = b {
            panic!();
        }
    }

    #[test]
    fn test_pawn_move_capture_false() {
        let g = Game::new();
        let p = g.get_piece_at(4, 1).unwrap();
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

        let p = g.get_piece_at(start.0, start.1).unwrap();
        let m = p.moves[2].prune(&g, start);

        if m.len() != 1 || !m.contains_key(&(goal.0 + goal.1 * 8)) {
            panic!();
        }

        println!("Move success: {}", g.make_move(start, goal));
        let a = g.get_piece_at(goal.0, goal.1);

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

        let a = g.get_piece_at(goal.0, goal.1);
        let b = g.get_piece_at(goal2.0, goal2.1);

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

        let p = g.get_piece_at(start.0, start.1).unwrap();
        let m = p.moves[2].prune(&g, start);

        if m.len() != 1 || !m.contains_key(&(goal.0 + goal.1 * 8)) {
            panic!();
        }

        println!("Move success: {}", g.make_move(start, goal));
        let a = g.get_piece_at(goal.0, goal.1);

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
        let p = g.get_piece_at(4, 6).unwrap();
        let m = p.moves[0].prune(&g, (4, 6));

        g.print_moves(4, 6);

        if m.len() != 1 || !m.contains_key(&(4 + 5 * 8)) {
            panic!();
        }
    }

    #[test]
    fn test_pawn_black_move_double() {
        let g = Game::new();
        let p = g.get_piece_at(4, 6).unwrap();
        let m = p.moves[1].prune(&g, (4, 6));

        g.print_moves(4, 6);

        if m.len() != 1 || !m.contains_key(&(4 + 4 * 8)) {
            panic!();
        }
    }

    #[test]
    fn test_pawn_black_move_capture_false() {
        let g = Game::new();
        let p = g.get_piece_at(4, 6).unwrap();
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

        let p = g.get_piece_at(4, 4).unwrap();
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

        let p = g.get_piece_at(3, 4).unwrap();
        let m = p.moves[2].prune(&g, (3, 4));

        if m.len() != 1 || !m.contains_key(&(4 + 3 * 8)) {
            panic!();
        }
    }

    #[test]
    fn test_promote() {
        let mut g = Game::new();
        g.just_move((2, 1), (2, 7));
        g.just_move((7, 6), (7, 7));

        g.print_board();

        print!("This move should fail: ");
        if g.make_move((1, 1), (1, 2)) {
            panic!("The move did not fail.");
        }

        print!("This move should fail: ");
        if g.make_move((1, 6), (1, 5)) {
            panic!("The move did not fail.");
        }

        if let Some((pos, piece)) = g.get_promotion() {
            if piece.rank != 'p' || piece.color != Color::White || pos != (2, 7) {
                panic!("Wrong piece or position.");
            }

            if g.promote(pos, 'p')
                || g.promote(pos, 'K')
                || g.promote((4, 6), 'Q')
                || g.promote((7, 7), 'Q')
            {
                g.print_board();
                panic!("It should not have promoted!");
            }

            if !g.promote(pos, 'Q') {
                panic!("It should have promoted, but did not.");
            }

            if let Some(p) = g.get_piece_at(pos.0, pos.1) {
                if p.rank != 'Q' {
                    panic!("It lied about promoting!");
                }
                println!("Success!");
            } else {
                panic!("It is just gone. What?");
            }
        } else {
            panic!("No piece to promote.");
        }
    }

    #[test]
    fn test_check() {
        let mut t = test_template;
        t[3 + 3 * 8] = 'R';
        let b = Game::make_board(t, color_template).unwrap();
        let mut g = Game {
            board: b,
            turn_owner: Color::White,
            turn_count: 1,
            game_state: GameState::Running,
        };

        g.make_move((3, 3), (6, 3));

        if g.make_move((6, 7), (6, 6)) {
            g.print_board();
            panic!();
        }

        if !g.make_move((6, 7), (7, 7)) {
            panic!();
        }
    }
}
