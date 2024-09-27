use std::collections::*;
use super::*;

mod move_mod;
use move_mod::*;

/// A chess piece
#[derive(Clone, Debug)]
pub struct Piece {
    /// King: K, Queen: Q, Rook: R, Bishop: B, Knight: N, Pawn: p (optional in commands)
    pub rank: char,
    pub color: Color,
    pub is_crucial: bool,
    pub can_promote: bool,
    pub last_moved: Option<u32>,
    pub times_moved: u32,
    pub moves: Vec<Move>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    Capture(Position),
    Move(Position, Position)
}

#[derive(Clone, Copy, Debug,PartialEq)]
pub enum Position {
    /// ALWAYS relative to the "owner" of the move. 
    Relative((i8,i8)),
    Global((u8,u8)),
}

pub fn position (pos: Position, rel: (u8,u8)) -> (u8,u8) {
    match pos { 
        Position::Global((col, row)) => (col,row),
        Position::Relative((r_col, r_row)) => ((r_col + rel.0 as i8) as u8, (r_row + rel.1 as i8) as u8)
    }
}

impl Piece {
    /// When setting up a board, use the following notation.
    /// King: 'K', Queen: 'Q', Rook: 'R', Bishop: 'B', Knight: 'N', Pawn: 'p', No Piece: '0'
    pub fn new(color: Color, rank: char) -> Piece {
        match rank {
            'K' => Piece::new_king(color),
            'Q' => Piece::new_queen(color),
            'B' => Piece::new_bishop(color),
            'N' => Piece::new_knight(color),
            'R' => Piece::new_rook(color),
            'p' => Piece::new_pawn(color),

            _ => panic!("Maybe do not make it crash when making a piece that does not exist.")
        }
    }

    pub fn all_possible_moves (&self, col: u8, row: u8, game: &Game) -> HashMap<u8,Vec<Effect>> {
        let mut all = HashMap::<u8,Vec<Effect>>::new();
        for m in &self.moves {
            let batch = m.prune(game, (col, row));
            for (key, val) in batch {

                if !game.is_safe_move((col, row), (key % 8, key >> 3), &val, self.color) {
                    continue;
                }
                
                // This should never cause a collision (emphasis on should)
                all.insert(key, val);
            }
        }
        all
    }

    pub fn get_danger_zone (&self, col: u8, row: u8, game: &Game) -> HashSet<u8> {
        let mut all = HashSet::<u8>::new();
        for m in &self.moves {
            if !m.can_capture {
                let mut capture_effect = false;
                for e in &m.effect {
                    match e {
                        Effect::Capture(_a) => {capture_effect = true; break;},
                        _ => {}
                    }
                }
                if !capture_effect {
                    continue;
                }
            }

            let batch = m.prune(game, (col, row));
            for (key, val) in batch {
                if m.can_capture {
                    all.insert(key);
                }
                for e in &m.effect {
                    match e {
                        Effect::Capture(p) => {
                            match p {
                                Position::Global(g) => { all.insert(g.0 + g.1 * 8); },
                                Position::Relative(r) => { all.insert(r.0 as u8 + col + (r.1 as u8 + row) * 8); }
                            }
                        }
                        _ => {},
                    }
                }
            }
        }
        all
    }

    pub fn new_pawn(color: Color) -> Piece {
        let mut moves = Vec::<Move>::with_capacity(4);
        let enemyC = match color {
            Color::Black => Color::White,
            Color::White => Color::Black
        };
        let mult: (i8,i8) = if color == Color::White {
            (1,0)
        } else {
            (-1,1)
        };

        // Move forwards:
        moves.push(Move{
            maximum_slide: Some(1),
            directions: vec![(0,1*mult.0)],
            can_capture: false,
            color,
            ..Default::default()
        });
        // Move Two steps:
        moves.push(Move{
            maximum_slide: Some(2),
            minimum_slide: 2,
            directions: vec![(0,1*mult.0)],
            can_capture: false,
            // Only when it has not moved before.
            requirements: vec![
            PieceStatus{
                relative_pos: Some((0,0)),
                rank: Some('p'),
                has_moved: Some((Comparator::Exactly, 0)),
                ..Default::default()
            }],
            color,
            ..Default::default()
        });
        // Capture
        moves.push(Move{
            maximum_slide: Some(1),
            directions: vec![(1,1*mult.0)],
            mirror: Some(Mirror::Horizontally),
            requirements: vec![
                PieceStatus {
                    relative_pos: Some((1,1*mult.0)),
                    color: Some(enemyC),
                    rank: Some('0'),
                    ..Default::default()
                }
            ],
            color,
            ..Default::default()
        });
        // En Passant
        moves.push(Move{
            maximum_slide: Some(1),
            directions: vec![(1,1*mult.0)],
            mirror: Some(Mirror::Horizontally),
            can_capture: false, // It can not capture in the traditional way.
            requirements: vec![
                PieceStatus {
                    rank: Some('p'),
                    board_pos: (None, Some((8*mult.1 + 4*mult.0) as u8)),
                    relative_pos: Some((1,0)),
                    has_moved: Some((Comparator::Exactly, 1)),
                    color: Some(Color::Black),
                    last_moved: Some(0),
                    ..Default::default()
                }
            ],
            color,
            effect: vec![
                Effect::Capture(Position::Relative((1,0)))
            ],
            ..Default::default()
        });

        Piece {
            color,
            can_promote: true,
            rank: 'p',
            last_moved: None,
            times_moved: 0,
            is_crucial: false,
            moves,
        }
    }

    fn new_rook (color: Color) -> Piece {
        Piece {
            color,
            is_crucial: false,
            can_promote: false,
            rank: 'R',
            last_moved: None,
            times_moved: 0,
            moves: vec![
                Move {
                    directions: vec![
                       (0,1), (1,0)
                    ],
                    mirror: Some(Mirror::VerAndHor),
                    color,
                    ..Default::default()
                }
            ]
        }
    }

    fn new_bishop (color: Color) -> Piece {
        Piece {
            color,
            is_crucial: false,
            can_promote: false,
            rank: 'B',
            last_moved: None,
            times_moved: 0,
            moves: vec![
                Move {
                    directions: vec![
                       (1,1)
                    ],
                    mirror: Some(Mirror::VerAndHor),
                    color,
                    ..Default::default()
                }
            ]
        }
    }

    fn new_knight (color: Color) -> Piece {
        Piece {
            color,
            is_crucial: false,
            can_promote: false,
            rank: 'N',
            last_moved: None,
            times_moved: 0,
            moves: vec![
                Move {
                    directions: vec![
                       (2,1), (1,2)
                    ],
                    mirror: Some(Mirror::VerAndHor),
                    color,
                    ..Default::default()
                }
            ]
        }
    }
    fn new_queen (color: Color) -> Piece {
        Piece {
            color,
            is_crucial: false,
            can_promote: false,
            rank: 'Q',
            last_moved: None,
            times_moved: 0,
            moves: vec![
                Move {
                    directions: vec![
                       (0,1), (1,1), (1,0)
                    ],
                    mirror: Some(Mirror::VerAndHor),
                    color,
                    ..Default::default()
                }
            ]
        }
    }

    fn new_king (color: Color) -> Piece {
        Piece {
            color,
            is_crucial: true,
            can_promote: false,
            rank: 'K',
            last_moved: None,
            times_moved: 0,
            moves: vec![
                Move {
                    maximum_slide: Some(1),
                    directions: vec![
                       (0,1), (1,1), (1,0)
                    ],
                    mirror: Some(Mirror::VerAndHor),
                    color,
                    ..Default::default()
                },
                // Castling, King side
                Move {
                    maximum_slide: Some(2),
                    minimum_slide: 2,
                    can_capture: false,
                    color: Color::White,
                    directions: vec![(1,0)],
                    safe_throughout: true,
                    requirements: vec![
                        PieceStatus {
                            relative_pos: Some((0,0)),
                            has_moved: Some((Comparator::Exactly, 0)),
                            ..Default::default()
                        },
                        PieceStatus {
                            relative_pos: Some((3,0)),
                            color: Some(color),
                            rank: Some('R'),
                            has_moved: Some((Comparator::Exactly, 0)),
                            ..Default::default()
                        }
                    ],
                    command: Some("O-O".to_owned()),
                    effect: vec![Effect::Move(Position::Relative((3,0)), Position::Relative((1,0)))],
                    ..Default::default()
                },
                // Castling, Queen side
                Move {
                    maximum_slide: Some(2),
                    minimum_slide: 2,
                    can_capture: false,
                    color: Color::White,
                    directions: vec![(-1,0)],
                    safe_throughout: true,
                    requirements: vec![
                        PieceStatus {
                            relative_pos: Some((0,0)),
                            has_moved: Some((Comparator::Exactly, 0)),
                            ..Default::default()
                        },
                        PieceStatus {
                            relative_pos: Some((-4,0)),
                            color: Some(color),
                            rank: Some('R'),
                            has_moved: Some((Comparator::Exactly, 0)),
                            ..Default::default()
                        }
                    ],
                    command: Some("O-O-O".to_owned()),
                    effect: vec![Effect::Move(Position::Relative((-4,0)), Position::Relative((-1,0)))],
                    ..Default::default()
                }
            ]
        }

        // Add castling
    }
}