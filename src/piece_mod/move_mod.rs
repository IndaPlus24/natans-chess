use std::collections::*;

use crate::{Color, Game, GameState};

use super::*;

#[derive(Clone, Copy, Debug)]
pub enum Comparator {
    MoreThan, // x > y
    AtLeast,  // x >= y
    Exactly,  // x == y
    AtMost,   // x <= y
    LessThan, // x < y
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Mirror {
    Vertically,
    Horizontally,
    VerAndHor,
}

#[derive(Clone, Copy, Debug)]
pub struct PieceStatus {
    pub board_pos: (Option<u8>, Option<u8>),
    pub relative_pos: Option<(i8, i8)>,
    /// None means it is empty. Some('0') means it can be any rank.
    pub rank: Option<char>,
    pub color: Option<Color>,
    pub has_moved: Option<(Comparator, u32)>,
    /// 0 is the most recent turn completed by the owner,
    /// -1 is the turn before, and so on.
    /// 1 is the first turn of the game, 2 is after that, and so on.
    pub last_moved: Option<i32>,
}

impl Default for PieceStatus {
    fn default() -> Self {
        PieceStatus {
            board_pos: (None, None),
            relative_pos: None,
            rank: None,
            color: None,
            has_moved: None,
            last_moved: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Move {
    /// Describes the number of times that a move can be repeated in the same direction.
    /// If None, then there is no limit to the sliding.
    pub maximum_slide: Option<u8>,
    /// If 0, then they will be able to go nowhere.
    pub minimum_slide: u8,
    pub directions: Vec<(i8, i8)>,
    pub can_capture: bool,
    /// The color of the piece that this move belongs to.
    pub color: Color,
    pub mirror: Option<Mirror>,
    pub requirements: Vec<PieceStatus>,
    /// If a move would need a command that does not follow the traditional format, describe it here.
    pub command: Option<String>,
    /// If a move would capture a piece without landing on its space, 
    /// or it would cause a different piece to move, then this is how you do it.
    pub effect: Vec<Effect>,
    /// Apparently, you are unable to castle when in check.
    pub safe_throughout: bool
}

impl Default for Move {
    fn default() -> Self {
        Move {
            maximum_slide: None,
            minimum_slide: 1,
            directions: Vec::new(),
            can_capture: true,
            color: Color::White,
            mirror: None,
            requirements: Vec::new(),
            command: None,
            effect: Vec::new(),
            safe_throughout: false
        }
    }
}

impl Move {
    // Make it do a hash map that includes all the extra effects
    pub fn prune(&self, game: &Game, pos: (u8, u8)) -> HashMap<u8,Vec<Effect>> {
        let mut valid = HashMap::<u8,Vec<Effect>>::new();

        if self.safe_throughout && game.is_safe_position(pos.0, pos.1, self.color) {
            return valid;
        }

        let p_col = pos.0;
        let p_row = pos.1;

        let max_s = match self.maximum_slide {
            Some(n) => n,
            _ => 8,
        };

        for di in &self.directions {
            let d_col = di.0;
            let d_row = di.1;

            if check_conditions(game, pos, &self.requirements, None) {
                for value in prune_dir(
                    p_col,
                    p_row,
                    d_col,
                    d_row,
                    self.minimum_slide,
                    max_s,
                    self.can_capture,
                    &self.color,
                    game,
                    self.safe_throughout
                ) {
                    valid.insert(value, self.effect.clone());
                }
            }

            // This is disguising.
            if let Some(m) = &self.mirror {
                if (*m == Mirror::Horizontally || *m == Mirror::VerAndHor)
                    && check_conditions(game, pos, &self.requirements, Some(Mirror::Horizontally))
                {
                    for value in prune_dir(
                        p_col,
                        p_row,
                        -d_col,
                        d_row,
                        self.minimum_slide,
                        max_s,
                        self.can_capture,
                        &self.color,
                        game,
                        self.safe_throughout
                    ) {
                        valid.insert(value, self.effect.clone());
                    }
                }
                if (*m == Mirror::Vertically || *m == Mirror::VerAndHor)
                    && check_conditions(game, pos, &self.requirements, Some(Mirror::Vertically))
                {
                    for value in prune_dir(
                        p_col,
                        p_row,
                        d_col,
                        -d_row,
                        self.minimum_slide,
                        max_s,
                        self.can_capture,
                        &self.color,
                        game,
                        self.safe_throughout
                    ) {
                        valid.insert(value, self.effect.clone());
                    }
                }
                if *m == Mirror::VerAndHor
                    && check_conditions(game, pos, &self.requirements, Some(Mirror::VerAndHor))
                {
                    for value in prune_dir(
                        p_col,
                        p_row,
                        -d_col,
                        -d_row,
                        self.minimum_slide,
                        max_s,
                        self.can_capture,
                        &self.color,
                        game,
                        self.safe_throughout
                    ) {
                        valid.insert(value, self.effect.clone());
                    }
                }
            }
        }
        valid
    }
}

fn check_conditions(
    game: &Game,
    pos: (u8, u8),
    conditions: &Vec<PieceStatus>,
    mirror: Option<Mirror>,
) -> bool {
    for con in conditions {
        // This is quite the wacky math to flip relative positions and board positions.
        // It just makes sense.
        let mut cdf = 1 as i8;
        let mut cf = 0 as u8;
        let mut rdf = 1 as i8;
        let mut rf = 0 as u8;

        if let Some(ref m) = mirror {
            if *m == Mirror::Horizontally || *m == Mirror::VerAndHor {
                cdf = -1;
                cf = 1;
            }
            if *m == Mirror::Vertically || *m == Mirror::VerAndHor {
                rdf = -1;
                rf = 1;
            }
        }

        let piece = match con.relative_pos {
            Some(r_pos) => {
                let col = (pos.0 as i8 + r_pos.0 * cdf) as u8;
                let row = (pos.1 as i8 + r_pos.1 * rdf) as u8;

                // If a board row or column is specified, they must match the relative position.
                if let Some(c) = con.board_pos.0 {
                    if col != ((8 * cf) as i8 + c as i8 * cdf) as u8 {
                        return false;
                    }
                }
                if let Some(r) = con.board_pos.1 {
                    if row != ((8 * rf) as i8 + r as i8 * rdf) as u8 {
                        return false;
                    }
                }

                game.piece_at(col, row)
            }

            _ => {
                // If relative position is not defined, then board position must be defined.
                let col = ((8 * cf) as i8 + con.board_pos.0.unwrap() as i8 * cdf) as u8;
                let row = ((8 * rf) as i8 + con.board_pos.1.unwrap() as i8 * rdf) as u8;

                game.piece_at(col, row)
            }
        };

        // If everything else is good, then just check if it matches.
        return check_piece_status(piece, con, game);
    }
    true
}

fn check_piece_status(piece: Option<&Piece>, status: &PieceStatus, game: &Game) -> bool {
    if let Some(p) = piece {
        // There is a piece
        if let Some(rank) = status.rank {
            // There is supposed to be a piece

            // Check rank
            if rank != '0' && rank != p.rank {
                return false;
            }

            // Check color
            if let Some(color) = &status.color {
                if *color != p.color {
                    // Wrong color (racism)
                    return false;
                }
            }

            // Check if the last move matches
            match status.last_moved {
                Some(last_move) if last_move > 0 => {
                    if p.last_moved == None || last_move as u32 != p.last_moved.unwrap() {return false;}
                }
                Some(last_move) if last_move <= 0 => {
                    if let Some(p_last_move) = p.last_moved {
                        // The math makes sense, I think.
                        let turn = game.turn_count as i32 + last_move
                            - if game.turn_owner == p.color || game.turn_owner == Color::White {
                                1
                            } else {
                                0
                            };
                        if turn != p_last_move as i32 {
                            return false;
                        }
                    } else {
                        // It has not moved ever, so it fails.
                        return false;
                    }
                }
                _ => {}    
            }

            // Check if it has moved the right amount of times
            if let Some(cv) = &status.has_moved {
                use Comparator::*;
                match cv.0 {
                    MoreThan => if p.times_moved <= cv.1 { return false; },
                    AtLeast => if p.times_moved < cv.1 { return false; },
                    Exactly => if p.times_moved != cv.1 { return false; },
                    AtMost => if p.times_moved > cv.1 { return false; },
                    LessThan => if p.times_moved >= cv.1 { return false; }
                }
            }
        } else {
            // Turns out, there is not supposed to be a piece here
            return false;
        }
    } else {
        // There is no piece
        if let Some(_s) = status.rank {
            // But there is supposed to be a piece here
            return false;
        }
    }
    true
}

fn prune_dir(
    p_col: u8,
    p_row: u8,
    d_col: i8,
    d_row: i8,
    min_s: u8,
    max_s: u8,
    can_capture: bool,
    color: &Color,
    game: &Game,
    safe_throughout: bool,
) -> Vec<u8> {
    let mut r = Vec::<u8>::new();

    for i in 0..=max_s {
        let col = p_col as i8 + i as i8 * d_col;
        let row = p_row as i8 + i as i8 * d_row;

        // Do not step outside the edge.
        if col < 0 || row < 0 || col >= 8 || row >= 8 {
            return r;
        }

        if safe_throughout && !game.is_safe_position(col as u8, row as u8, *color) {
            return r;
        }

        let p = game.piece_at(col as u8, row as u8);

        match p {
            None => if i >= min_s { r.push(col as u8 + row as u8 * 8)},
            Some(piece) => {
                if ((can_capture && piece.color != *color) || i == 0) && i >= min_s{
                    r.push(col as u8 + row as u8 * 8);
                }
                // Do not collide with yourself
                if i > 0 {
                    return r;
                }
            }
        }
    }

    r
}
