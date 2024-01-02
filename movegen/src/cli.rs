use std::collections::VecDeque;

use clap::{Parser, Subcommand};
use util::square::Square;

use crate::{board::Board, mv::Move};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
#[clap(rename_all = "snake_case")]
pub enum Command {
    Interactive,
    Position {
        fen: String,
    },
    Perft {
        ply: u8,
        fen: Option<String>,
        #[arg(short, long)]
        thread_count: Option<usize> 
    },
    Play {
        ply: u8,

        #[arg(short, long, default_value_t = 250)]
        frame_time: u64,
    },
    Move {
        mv: String,
    },
    Moves {
        sq: Option<String>,
    },
    Print,
    Exit,
    Reset,
}

pub fn handle_command(cmd: Command, board: &mut Board) {
    match cmd {
        Command::Perft { ply, fen, thread_count } => {
            if let Some(fen) = fen {
                *board = match Board::from_fen(&fen) {
                    Ok(b) => b,
                    Err(err) => {
                        println!("Error: {:?}", err);
                        return;
                    }
                }
            }
            if let Some(thread_count) = thread_count {
                board.perft_multithread(ply, thread_count);
            } else {
                board.perft(ply);
            }
        }
        Command::Play { ply, frame_time } => {
            board.play(ply, frame_time);
        }
        Command::Reset => *board = Board::default(),
        Command::Print => {
            board.pretty_print(false);
        }
        Command::Position { fen } => {
            *board = match Board::from_fen(&fen) {
                Ok(board) => board,
                Err(err) => {
                    println!("Error: {:?}", err);
                    return;
                }
            }
        }
        Command::Move { mv } => {
            let mv = match Move::from_str(&mv, board) {
                Ok(mv) => mv,
                Err(err) => {
                    println!("Error: {:?}", err);
                    return;
                }
            };
            board.make_move(&mv);
        }
        Command::Moves { sq } => {
            let moves = match sq {
                Some(sq) => match Square::from_str(&sq) {
                    Ok(sq) => board.get_sq_moves(sq),
                    Err(err) => {
                        println!("Error: {:?}", err);
                        return;
                    }
                },
                None => board.get_moves(),
            };
            println!(
                "Moves: {}",
                moves.iter().fold(String::new(), |mut acc, mv| {
                    acc += &mv.to_string();
                    acc += ", ";
                    acc
                })
            )
        }
        _ => {}
    }
}

pub fn split_ignore_quotes(input: &str) -> VecDeque<String> {
    let mut split_strings = VecDeque::new();
    let mut inside_quotes = false;
    let mut current_string = String::new();

    for c in input.chars() {
        match c {
            '\'' => inside_quotes = !inside_quotes,
            ' ' if !inside_quotes => {
                if !current_string.is_empty() {
                    split_strings.push_back(current_string.clone());
                    current_string.clear();
                }
            }
            _ => current_string.push(c),
        }
    }

    if !current_string.is_empty() {
        split_strings.push_back(current_string.clone());
    }

    split_strings
}
