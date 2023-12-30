use crate::{bitboard::Bitboard, helper::impl_conv, square::File};

impl File {
    const A_BB: Bitboard = Bitboard(0x0101010101010101);
    const B_BB: Bitboard = Bitboard(Self::A_BB.0 << 1);
    const C_BB: Bitboard = Bitboard(Self::B_BB.0 << 1);
    const D_BB: Bitboard = Bitboard(Self::C_BB.0 << 1);
    const E_BB: Bitboard = Bitboard(Self::D_BB.0 << 1);
    const F_BB: Bitboard = Bitboard(Self::E_BB.0 << 1);
    const G_BB: Bitboard = Bitboard(Self::F_BB.0 << 1);
    const H_BB: Bitboard = Bitboard(Self::G_BB.0 << 1);
    pub fn bitboard(&self) -> Bitboard {
        match self {
            Self::A => Self::A_BB,
            Self::B => Self::B_BB,
            Self::C => Self::C_BB,
            Self::D => Self::D_BB,
            Self::E => Self::E_BB,
            Self::F => Self::F_BB,
            Self::G => Self::G_BB,
            Self::H => Self::H_BB,
        }
    }
}

impl_conv! {
    File,
    char,
    from_char,
    to_char,
    'a'=A,
    'b'=B,
    'c'=C,
    'd'=D,
    'e'=E,
    'f'=F,
    'g'=G,
    'h'=H
}
