use crate::{bitboard::Bitboard, color::Color, error::ChessError};

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
