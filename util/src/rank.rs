use crate::{bitboard::Bitboard, color::Color, error::ChessError, helper::impl_conv, square::Rank};

impl Rank {
    const FIRST: Bitboard = Bitboard(0xFF);
    const SECOND: Bitboard = Bitboard(Self::FIRST.0 << 8);
    const THIRD: Bitboard = Bitboard(Self::SECOND.0 << 8);
    const FOURTH: Bitboard = Bitboard(Self::THIRD.0 << 8);
    const FIFTH: Bitboard = Bitboard(Self::FOURTH.0 << 8);
    const SIXTH: Bitboard = Bitboard(Self::FIFTH.0 << 8);
    const SEVENTH: Bitboard = Bitboard(Self::SIXTH.0 << 8);
    const EIGHTH: Bitboard = Bitboard(Self::SEVENTH.0 << 8);

    pub fn bitboard(&self) -> Bitboard {
        match self {
            Self::First => Self::FIRST,
            Self::Second => Self::SECOND,
            Self::Third => Self::THIRD,
            Self::Fourth => Self::FOURTH,
            Self::Fifth => Self::FIFTH,
            Self::Sixth => Self::SIXTH,
            Self::Seventh => Self::SEVENTH,
            Self::Eighth => Self::EIGHTH,
        }
    }

    pub const fn pov(&self, color: &Color) -> Self {
        match color {
            Color::White => *self,
            Color::Black => Self::from_u8(7 - *self as u8),
        }
    }
}

impl_conv! {
    Rank,
    char,
    from_char,
    to_char,
    '1'=First,
    '2'=Second,
    '3'=Third,
    '4'=Fourth,
    '5'=Fifth,
    '6'=Sixth,
    '7'=Seventh,
    '8'=Eighth
}
