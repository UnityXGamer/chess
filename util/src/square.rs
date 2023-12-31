use crate::{bitboard::Bitboard, color::Color, error::ChessError, helper::impl_conv};

chess_macro::make_ranks_files_squares!();

impl Square {
    pub fn from_str(str: &str) -> Result<Self, ChessError> {
        let mut chars = str.chars();

        let (c1, c2) = match (chars.next(), chars.next(), chars.next()) {
            (Some(c1), Some(c2), None) => (c1, c2),
            _ => {
                return Err(ChessError::Parse(format!(
                    "'{str}' cannot be used to construct a square"
                )))
            }
        };

        let file = match File::from_char(c1) {
            Some(f) => f,
            None => {
                return Err(ChessError::Parse(format!(
                    "Char '{c1}' cannot be used to create a file"
                )))
            }
        };

        let rank = match Rank::from_char(c2) {
            Some(r) => r,
            None => {
                return Err(ChessError::Parse(format!(
                    "Char '{c2}' cannot be used to create a rank"
                )))
            }
        };

        Ok(Self::from_rank_file(rank, file))
    }
    pub const fn rank(&self) -> Rank {
        Rank::from_u8(*self as u8 / 8)
    }
    pub const fn file(&self) -> File {
        File::from_u8(*self as u8 % 8)
    }
    pub fn to_string(&self) -> String {
        format!("{}{}", self.file().to_char(), self.rank().to_char())
    }
    pub const fn from_rank_file(rank: Rank, file: File) -> Self {
        Self::from_u8(rank as u8 * 8 + file as u8)
    }
    pub const fn apply_delta(&self, (d_rank, d_file): (i8, i8)) -> Option<Self> {
        if let Some(file) = self.file().increment_checked(d_file) {
            if let Some(rank) = self.rank().increment_checked(d_rank) {
                return Some(Self::from_rank_file(rank, file));
            }
        }
        None
    }
    pub fn ep_move_sq(active_color: &Color, file: File) -> Self {
        Self::from_rank_file(Rank::Third.pov(active_color), file)
    }
    pub fn ep_pawn_sq(active_color: &Color, file: File) -> Self {
        Self::from_rank_file(Rank::Fourth.pov(active_color), file)
    }
    pub const fn idx(&self) -> usize {
        *self as usize
    }
    pub const fn bitboard(&self) -> Bitboard {
        Bitboard(1 << *self as u8)
    }
}

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
