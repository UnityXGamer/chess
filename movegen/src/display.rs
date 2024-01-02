use crate::{color::Color, piece::Piece};

pub fn display_piece(color: &Color, kind: &Piece) -> char {
    match (color, kind) {
        (Color::White, Piece::Pawn) => 'P',
        (Color::White, Piece::Knight) => 'N',
        (Color::White, Piece::Bishop) => 'B',
        (Color::White, Piece::Rook) => 'R',
        (Color::White, Piece::Queen) => 'Q',
        (Color::White, Piece::King) => 'K',
        (Color::Black, Piece::Pawn) => 'p',
        (Color::Black, Piece::Knight) => 'n',
        (Color::Black, Piece::Bishop) => 'b',
        (Color::Black, Piece::Rook) => 'r',
        (Color::Black, Piece::Queen) => 'q',
        (Color::Black, Piece::King) => 'k',
    }
}
