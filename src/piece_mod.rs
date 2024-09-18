use crate::Color;

mod move_mod;
use move_mod::*;

/// A chess piece
pub struct Piece {
    /// King: K, Queen: Q, Rook: R, Bishop: B, Knight: N, Pawn: p (optional in commands), Nothing: 0
    pub rank: char,
    pub color: Color,
    pub is_crucial: bool,
    pub can_promote: bool,
    pub last_moved: Option<u32>,
    pub times_moved: u32,
    pub moves: Vec<Move>,
}

impl Piece {
    pub fn new(color: Color, rank: char) -> Piece {
        match rank {
            'r' => Piece::new_pawn(color),

            _ => panic!("Maybe do not make it crash when making a piece that does not exist.")
        }
    }
    pub fn new_pawn(color: Color) -> Piece {
        let mut moves = Vec::<Move>::with_capacity(4);
        let anemyC = match color {
            Color::Black => Color::White,
            Color::White => Color::Black
        };
        // Move forwards:
        moves.push(Move{
            maximum_slide: Some(1),
            directions: vec![(0,1)],
            can_capture: false,
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
                    color: Some(Color::Black),
                    ..Default::default()
                }
            ],
            ..Default::default()
        });
        // En Passant
        moves.push(Move{
            maximum_slide: Some(1),
            directions: vec![(1,1)],
            mirror: Some(Mirror::Vertically),
            requirements: vec![
                PieceStatus {
                    rank: Some('p'),
                    board_pos: panic!("You forgot to figure out how the board position works!"),
                    relative_pos: Some((1,0)),
                    has_moved: Some((Comparator::Exactly, 1)),
                    color: Some(Color::Black),
                    last_moved: Some(0),
                    ..Default::default()
                }
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
            moves: Vec::new(),
        }
    }
}