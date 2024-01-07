#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub const ALL: [Self; 2] = [Self::White, Self::Black];
    pub fn from_char(c: char) -> Self {
        if c.is_uppercase() {
            Self::White
        } else {
            Self::Black
        }
    }
    pub fn to_fen(&self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b'
        }
    }
}

impl std::ops::Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl std::ops::Not for &Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Color::White => &Color::Black,
            Color::Black => &Color::White,
        }
    }
}
