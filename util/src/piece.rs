use crate::helper::impl_conv;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub const ALL: [Self; 6] = [
        Self::Pawn,
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
        Self::King,
    ];
}

impl_conv! {
    Piece,
    char,
    from_char,
    to_char,
    'p'=Pawn,
    'n'=Knight,
    'b'=Bishop,
    'r'=Rook,
    'q'=Queen,
    'k'=King
}
