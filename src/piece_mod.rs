use std::default;

use crate::Color;

mod move_mod;
use move_mod::*;

/// A chess piece
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

impl Piece {
    /// When setting up a board, use the following notation.
    /// King: 'K', Queen: 'Q', Rook: 'R', Bishop: 'B', Knight: 'N', Pawn: 'p', No Piece: '0'
    pub fn new(color: Color, rank: char) -> Piece {
        match rank {
            'K' => Piece::new_king(color),
            'Q' => Piece::new_queen(color),
            'B' => Piece::new_bishop(color),
            'N' => Piece::new_king(color),
            'R' => Piece::new_rook(color),
            'p' => Piece::new_pawn(color),

            _ => panic!("Maybe do not make it crash when making a piece that does not exist.")
        }
    }
    pub fn new_pawn(color: Color) -> Piece {
        let mut moves = Vec::<Move>::with_capacity(4);
        let enemyC = match color {
            Color::Black => Color::White,
            Color::White => Color::Black
        };
        // Move forwards:
        moves.push(Move{
            maximum_slide: Some(1),
            directions: vec![(0,1)],
            can_capture: false,
            color,
            ..Default::default()
        });
        // Move Two steps:
        moves.push(Move{
            maximum_slide: Some(2),
            minimum_slide: 2,
            directions: vec![(0,1)],
            can_capture: false,
            // Only when it has not moved before.
            requirements: vec![
            PieceStatus{
                relative_pos: Some((0,0)),
                has_moved: Some((Comparator::Exactly, 0)),
                ..Default::default()
            }],
            color,
            ..Default::default()
        });
        // Capture
        moves.push(Move{
            maximum_slide: Some(1),
            directions: vec![(1,1)],
            mirror: Some(Mirror::Vertically),
            requirements: vec![
                PieceStatus {
                    relative_pos: Some((1,1)),
                    color: Some(enemyC),
                    ..Default::default()
                }
            ],
            color,
            ..Default::default()
        });
        // En Passant
        moves.push(Move{
            maximum_slide: Some(1),
            directions: vec![(1,1)],
            mirror: Some(Mirror::Vertically),
            can_capture: false, // It can not capture in the traditional way.
            requirements: vec![
                PieceStatus {
                    rank: Some('p'),
                    board_pos: (None, Some(4)),
                    relative_pos: Some((1,0)),
                    has_moved: Some((Comparator::Exactly, 1)),
                    color: Some(Color::Black),
                    last_moved: Some(0),
                    ..Default::default()
                }
            ],
            color,
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
                }
            ]
        }

        // Add castling
    }
}