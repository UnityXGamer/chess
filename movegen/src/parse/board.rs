use std::{fmt::{Write, format}, io::Empty};

use util::{
    bitboard::Bitboard,
    color::Color,
    error::ChessError,
    piece::Piece,
    square::{File, Rank, Square},
};

use crate::{
    board::Board,
    both_colors::BothColors,
    piece_bb::PieceBitboards,
    state::{Castling, State},
};

impl Board {
    pub fn fen(&self, url_encode: bool) -> String {
        let mut fen = String::new();
        for rank in self.all_sqs(true) {
            let mut empty_number = 0;
            for (_, piece) in rank {
                if let Some((color, piece)) = piece {
                    if empty_number > 0 {
                        fen += &format!("{empty_number}");
                        empty_number = 0;
                    }
                    let mut char = piece.to_char();
                    if color == Color::White {
                        char.make_ascii_uppercase()
                    }
                    fen += &format!("{char}");
                } else {
                    empty_number += 1;
                }
            }
            if empty_number > 0 {
                fen += &format!("{empty_number}");
            }
            fen += "/"
        }
        if fen.ends_with("/") {
            fen.pop();
        }

        fen += &format!(" {}", self.state.active_color.to_fen());
        fen += &format!(" {}", self.state.castling.to_fen());
        fen += &format!(" {}", if let Some(ep_file) = self.state.ep_file {
            Square::ep_move_sq(&self.state.active_color, ep_file).to_string()
        } else {
            "-".to_string()
        });
        fen+=&format!(" {}", self.state.half_move_count);
        fen+=&format!(" {}", self.state.full_move_count);
        
        if url_encode {
            fen = fen.replace(" ", "_");
        }

        fen
    }
    /// see https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
    pub fn from_fen(input: &str) -> Result<Self, ChessError> {
        let mut fen_chunks: [&str; 6] = [""; 6];

        for (i, c) in input.split(" ").enumerate() {
            if i <= 5 {
                fen_chunks[i] = c;
            } else {
                return Err(ChessError::Parse(format!("Too many FEN segments")));
            }
        }

        if fen_chunks[3] == "" {
            return Err(ChessError::Parse(format!("Too few FEN segments")));
        }

        // These are not provided in some test FEN:s
        if fen_chunks[4] == "" {
            fen_chunks[4] = "0"
        }

        if fen_chunks[5] == "" {
            fen_chunks[5] = "1"
        }

        let pieces = match BothColors::from_fen(fen_chunks[0]) {
            Ok(pieces) => pieces,
            Err(err) => return Err(err),
        };

        let state = match State::from_fen(fen_chunks) {
            Ok(state) => state,
            Err(err) => return Err(err),
        };

        let mut board = Self {
            pinned: Bitboard::EMPTY,
            check_masks: [None, None],
            pieces,
            state,
        };

        board.update_slider_checks_pins(&!board.state.active_color);

        Ok(board)
    }
}

impl BothColors<PieceBitboards> {
    fn from_fen(fen_chunk: &str) -> Result<Self, ChessError> {
        let mut rank = Rank::Eighth;

        let ranks = fen_chunk.split("/");
        let mut bitboards = Self::default();

        for rank_str in ranks {
            let mut file = File::A;
            let mut last_char_number = None;
            for rank_char in rank_str.chars() {
                let too_many_sqs = ChessError::Parse(format!(
                    "Rank {:?} ({rank_str}) empty squares are too large",
                    rank
                ));
                if let Ok(number) = rank_char.to_string().parse::<u8>() {
                    if let Some(_) = last_char_number {
                        return Err(ChessError::Parse(format!(
                            "Rank {:?} ({rank_str}) has two adjacent empty square numbers",
                            rank
                        )));
                    };
                    last_char_number = Some(number);
                    if let Some(next_file) = file.increment_checked(number as i8 - 1) {
                        file = next_file;
                    } else {
                        return Err(too_many_sqs);
                    }
                } else {
                    let (color, piece) = &match (
                        Color::from_char(rank_char),
                        Piece::from_char(rank_char.to_ascii_lowercase()),
                    ) {
                        (c, Some(piece)) => (c, piece),
                        _ => {
                            return Err(ChessError::Parse(format!(
                                "'{rank_char}' cannot be parsed as a piece"
                            )))
                        }
                    };
                    let sq = Square::from_rank_file(rank, file);
                    bitboards[color][piece] |= sq.bitboard();
                    bitboards[color].all |= sq.bitboard();
                    last_char_number = None;
                }

                if file == File::H {
                    continue;
                } else if let Some(next_file) = file.increment_checked(1) {
                    file = next_file;
                } else {
                    return Err(too_many_sqs);
                }
            }
            if file != File::H {
                return Err(ChessError::Parse(format!(
                    "Rank {:?} ({rank_str}) has too few squares",
                    rank
                )));
            }
            if let Some(next_rank) = rank.increment_checked(-1) {
                rank = next_rank
            }
        }
        Ok(bitboards)
    }
}

impl State {
    fn from_fen(fen_chunks: [&str; 6]) -> Result<Self, ChessError> {
        let active_color = match fen_chunks[1] {
            "w" => Color::White,
            "b" => Color::Black,
            other => {
                return Err(ChessError::Parse(format!(
                    "'{other}' cannot be used to construct active_color"
                )))
            }
        };

        let castling = match BothColors::from_str(fen_chunks[2]) {
            Ok(castling) => castling,
            Err(err) => return Err(err),
        };

        let ep_file = match fen_chunks[3] {
            "-" => None,
            sq => match Square::from_str(sq) {
                Ok(sq) => Some(sq.file()),
                Err(err) => return Err(err),
            },
        };

        let half_move_count = match fen_chunks[4].parse::<u16>() {
            Ok(n) => n,
            Err(_) => {
                return Err(ChessError::Parse(format!(
                    "Half move count could not be parsed from '{val}'",
                    val = fen_chunks[4]
                )))
            }
        };

        let full_move_count = match fen_chunks[5].parse::<u16>() {
            Ok(n) => n,
            Err(_) => {
                return Err(ChessError::Parse(format!(
                    "Full move count could not be parsed from '{val}'",
                    val = fen_chunks[5]
                )))
            }
        };

        Ok(Self {
            active_color,
            castling,
            ep_file,
            half_move_count,
            full_move_count,
        })
    }
}

impl BothColors<Castling> {
    fn from_str(input: &str) -> Result<Self, ChessError> {
        let mut castling = Self::default();
        let w = &Color::White;
        let b = &Color::Black;
        let err = ChessError::Parse(format!("Castling input '{input}' is invalid"));
        for i in input.char_indices() {
            match i {
                (0, '-') => return Ok(castling),
                (_, '_') => return Err(err),
                (_, 'K') => castling[w].king_side = true,
                (_, 'Q') => castling[w].queen_side = true,
                (_, 'k') => castling[b].king_side = true,
                (_, 'q') => castling[b].queen_side = true,
                _ => return Err(err),
            }
        }
        Ok(castling)
    }
    fn to_fen(&self) -> String {
        let mut output = String::new();
        if self[&Color::White].king_side {
            output += "K"
        };
        if self[&Color::White].king_side {
            output += "Q"
        };
        if self[&Color::White].king_side {
            output += "k"
        };
        if self[&Color::White].king_side {
            output += "q"
        };
        if output.is_empty() {
            "-".to_string()
        } else {
            output
        }
    }
}
