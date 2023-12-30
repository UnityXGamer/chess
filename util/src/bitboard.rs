use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

use chess_macro::make_bitboard;

use crate::square::{File, Rank, Square};

use crate::color::Color;

impl Iterator for Bitboard {
    type Item = Square;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_sq()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const FULL: Self = Self(u64::MAX);
    pub const EMPTY: Self = Self(0);
    pub const EDGES: Self = make_bitboard!(
        X X X X X X X X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X X X X X X X X
    );

    pub fn sq_count(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn subsets(&self) -> Vec<Self> {
        let mut s = Self::EMPTY;
        let mut bbs = Vec::with_capacity(1 << self.0.count_ones());
        // see https://www.chessprogramming.org/Traversing_Subsets_of_a_Set#All_Subsets_of_any_Set
        loop {
            bbs.push(s);
            s.0 = s.0.wrapping_sub(self.0) & self.0;
            if s.is_empty() {
                return bbs;
            }
        }
    }
    pub fn next_sq(&mut self) -> Option<Square> {
        if !self.is_empty() {
            let idx = self.0.trailing_zeros() as u8;
            self.0 ^= 1 << idx;
            Square::from_u8_checked(idx)
        } else {
            None
        }
    }
    pub const fn is_empty(&self) -> bool {
        self.0 == Self::EMPTY.0
    }
    pub fn has_sq(&self, sq: Square) -> bool {
        !(*self & sq.bitboard()).is_empty()
    }
    pub fn increase_rank(&self, active_color: &Color) -> Self {
        Self(match active_color {
            Color::White => self.0 << 8,
            Color::Black => self.0 >> 8,
        })
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n")?;
        for rank in Rank::ALL.iter().rev() {
            write!(f, "\n")?;
            for file in File::ALL {
                if self.has_sq(Square::from_rank_file(*rank, file)) {
                    write!(f, "x")?
                } else {
                    write!(f, ".")?
                }
            }
        }
        Ok(())
    }
}

macro_rules! impl_bitwise {
    ($t:ident, $f:ident) => {
        impl $t for Bitboard {
            type Output = Self;
            fn $f(self, rhs: Self) -> Self::Output {
                Self($t::$f(self.0, rhs.0))
            }
        }
    };
}

impl_bitwise! {BitAnd, bitand}
impl_bitwise! {BitOr, bitor}
impl_bitwise! {BitXor, bitxor}

macro_rules! impl_bitwise_assign {
    ($t:ident, $f:ident) => {
        impl $t for Bitboard {
            fn $f(&mut self, rhs: Self) {
                $t::$f(&mut self.0, rhs.0)
            }
        }
    };
}

impl_bitwise_assign! {BitAndAssign, bitand_assign}
impl_bitwise_assign! {BitOrAssign, bitor_assign}
impl_bitwise_assign! {BitXorAssign, bitxor_assign}

impl Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
