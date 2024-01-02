use util::{
    piece::Piece,
    square::Square, error::ChessError,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveFlag {
    None,
    Capture(Piece),
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
    pub fn from_str(input: &str) -> Result<Self, ChessError> {
        match input {
            "n" => Ok(Self::Knight),
            "b" => Ok(Self::Bishop),
            "r" => Ok(Self::Rook),
            "q" => Ok(Self::Queen),
            _ => Err(ChessError::Parse(format!("'{input}' cannot be used to construct a promotion")))
        }
    }
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
