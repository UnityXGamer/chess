use util::{error::ChessError, square::Square};

use crate::{board::Board, mv::Move};

impl Move {
    pub fn from_str(input: &str, board: &mut Board) -> Result<Self, ChessError> {
        if input.len() != 4 {
            return Err(ChessError::Parse(format!(
                "Move input should be of form <start><end> eg. 'e2e4'"
            )));
        }
        let piece_sq = match Square::from_str(&input[0..2]) {
            Ok(p) => p,
            Err(e) => return Err(e),
        };
        let to = match Square::from_str(&input[2..4]) {
            Ok(to) => to,
            Err(err) => return Err(err),
        };

        for mv in board.get_sq_moves(piece_sq) {
            if mv.to == to {
                return Ok(mv);
            }
        }

        Err(ChessError::InvalidMove(format!(
            "{input} is not a valid move",
        )))
    }
}
