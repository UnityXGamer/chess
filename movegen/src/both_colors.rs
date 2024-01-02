use std::ops::{Index, IndexMut};

use util::{bitboard::Bitboard, color::Color};

use crate::{piece_bb::PieceBitboards, state::Castling};

#[derive(Debug, Clone, Copy)]
pub struct BothColors<T> {
    black: T,
    white: T,
}

impl Default for BothColors<Castling> {
    fn default() -> Self {
        Self {
            white: Castling::default(),
            black: Castling::default(),
        }
    }
}

impl Default for BothColors<Bitboard> {
    fn default() -> Self {
        Self {
            white: Bitboard::EMPTY,
            black: Bitboard::EMPTY,
        }
    }
}

impl Default for BothColors<PieceBitboards> {
    fn default() -> Self {
        Self {
            white: PieceBitboards::default(),
            black: PieceBitboards::default(),
        }
    }
}

impl<T> Index<&Color> for BothColors<T> {
    type Output = T;
    fn index(&self, index: &Color) -> &Self::Output {
        match index {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }
}

impl<T> IndexMut<&Color> for BothColors<T> {
    fn index_mut(&mut self, index: &Color) -> &mut Self::Output {
        match index {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }
}
