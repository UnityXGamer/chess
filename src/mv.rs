use util::{
    bitboard::Bitboard,
    piece::Piece,
    square::{File, Square},
};

use crate::{both_colors::BothColors, state::Castling};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveFlag {
    None,
    Capture(Piece),
    /// stores the old ep_file
    PawnFirstMove,
    KingSideCastles,
    QueenSideCastles,
    EnPassant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Promotion {
    Knight,
    Bishop,
    Rook,
    Queen,
}

impl Promotion {
    pub const ALL: [Self; 4] = [Self::Knight, Self::Bishop, Self::Rook, Self::Queen];
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub piece: Piece,
    pub from: Square,
    pub to: Square,
    pub flag: MoveFlag,
    pub promotion: Option<Promotion>,
    pub old_castling: BothColors<Castling>,
    pub old_ep_file: Option<File>,
}

impl Move {
    pub fn to_string(&self) -> String {
        let mut s = self.from.to_string() + &self.to.to_string();
        match self.promotion {
            Some(Promotion::Knight) => s += "n",
            Some(Promotion::Bishop) => s += "b",
            Some(Promotion::Rook) => s += "r",
            Some(Promotion::Queen) => s += "q",
            None => {}
        };
        s
    }
}
