use std::collections::HashSet;

use crate::{Color, Game};

use super::Piece;

pub enum Comparator {
    MoreThan, // x > y
    AtLeast,  // x >= y
    Exactly,  // x == y
    AtMost,   // x <= y
    LessThan, // x < y
}

#[derive(PartialEq, Eq)]
pub enum Mirror {
    Vertically,
    Horizontally,
    VerAndHor,
}


pub struct PieceStatus {
    pub board_pos: (Option<u8>, Option<u8>),
    pub relative_pos: Option<(i8, i8)>,
    /// None means there has to be a piece of any rank. Some('0') means it has to be empty.
    pub rank: Option<char>,
    pub color: Option<Color>,
    pub has_moved: Option<(Comparator, u8)>,
    /// 0 is "last turn",
    /// -1 is the turn before and -2 is before that,
    /// 1 is the first turn of the game and 2 is after that
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

pub struct Move {
    /// Describes the number of times that a move can be repeated in the same direction.
    /// If None, then there is no limit to the sliding.
    pub maximum_slide: Option<u8>,
    /// If 0, then they will be able to go nowhere
    pub minimum_slide: u8,
    pub directions: Vec<(i8, i8)>,
    pub can_capture: bool,
    pub mirror: Option<Mirror>,
    pub requirements: Vec<PieceStatus>,
}

impl Default for Move {
    fn default() -> Self {
        Move {
            maximum_slide: None,
            minimum_slide: 1,
            directions: Vec::new(),
            can_capture: true,
            mirror: None,
            requirements: Vec::new(),
        }
    }
}

impl Move {
    pub fn prune(&self, game: &Game, pos: (u8, u8)) -> HashSet<u8> {
        let mut valid = HashSet::<u8>::new();

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
                    game,
                ) {
                    valid.insert(value);
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
                        game,
                    ) {
                        valid.insert(value);
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
                        game,
                    ) {
                        valid.insert(value);
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
                        game,
                    ) {
                        valid.insert(value);
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

        if let Some(r_pos) = con.relative_pos {
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

            let p = game.piece_at(col, row);

            // Compare the piece and piece status here
        } else {
            // We need a position, so board position must be fully defined.
            let col = ((8 * cf) as i8 + con.board_pos.0.unwrap() as i8 * cdf) as u8;
            let row = ((8 * rf) as i8 + con.board_pos.1.unwrap() as i8 * rdf) as u8;

            let p = game.piece_at(col, row);

            // Compare the piece and piece status here
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
    game: &Game,
) -> Vec<u8> {
    let mut r = Vec::<u8>::new();

    for i in min_s..=max_s {
        let col = p_col as i8 + i as i8 * d_col;
        let row = p_row as i8 + i as i8 * d_row;

        // Do not step outside the edge.
        if col < 0 || row < 0 || col >= 8 || row >= 8 {
            return r;
        }

        let p = game.piece_at(col as u8, row as u8);

        match p {
            None => r.push(col as u8 * 8 + row as u8),
            _ => {
                if can_capture {
                    r.push(col as u8 * 8 + row as u8)
                }
                return r;
            }
        }
    }

    r
}
