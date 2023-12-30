#[derive(Debug)]
pub enum ChessError {
    Parse(String),
    InvalidPieceAccess,
    InvalidMove(String),
}
