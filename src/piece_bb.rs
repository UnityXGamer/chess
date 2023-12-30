use std::ops::{Index, IndexMut};
use util::{bitboard::Bitboard, piece::Piece};

#[derive(Debug, Clone)]
pub struct PieceBitboards {
    pub all: Bitboard,
    pub pawn: Bitboard,
    pub knight: Bitboard,
    pub bishop: Bitboard,
    pub rook: Bitboard,
    pub queen: Bitboard,
    pub king: Bitboard,
}

impl Default for PieceBitboards {
    fn default() -> Self {
        Self {
            all: Bitboard::EMPTY,
            pawn: Bitboard::EMPTY,
            knight: Bitboard::EMPTY,
            bishop: Bitboard::EMPTY,
            rook: Bitboard::EMPTY,
            queen: Bitboard::EMPTY,
            king: Bitboard::EMPTY,
        }
    }
}

impl Index<&Piece> for PieceBitboards {
    type Output = Bitboard;
    fn index(&self, index: &Piece) -> &Self::Output {
        match index {
            Piece::Pawn => &self.pawn,
            Piece::Knight => &self.knight,
            Piece::Bishop => &self.bishop,
            Piece::Rook => &self.rook,
            Piece::Queen => &self.queen,
            Piece::King => &self.king,
        }
    }
}

impl IndexMut<&Piece> for PieceBitboards {
    fn index_mut(&mut self, index: &Piece) -> &mut Self::Output {
        match index {
            Piece::Pawn => &mut self.pawn,
            Piece::Knight => &mut self.knight,
            Piece::Bishop => &mut self.bishop,
            Piece::Rook => &mut self.rook,
            Piece::Queen => &mut self.queen,
            Piece::King => &mut self.king,
        }
    }
}
