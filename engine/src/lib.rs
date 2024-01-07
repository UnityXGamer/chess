use movegen::{mv::Move, board::Board};
pub trait MoveSearch {
    type Input;
    fn init(input: Self::Input) -> Self;
    fn search(&mut self, board: &mut Board, best_move: &mut Option<Move>, ms_remaining: u64);
}


#[derive(Debug, Clone, Copy)]
pub struct Engine<T: MoveSearch> {
    pub best_move: Option<Move>,
    pub board: Board,
    move_searcher: T
}

impl<T: MoveSearch> Engine<T> {
    pub fn new(board: Board, input: T::Input) -> Self {
        Self {
            best_move: None,
            board,
            move_searcher: T::init(input),
        }
    }
    pub fn search(&mut self, ms_remaining: u64) {
        self.best_move = None;
        T::search(&mut self.move_searcher, &mut self.board, &mut self.best_move, ms_remaining)
    }
}

mod random;
pub use random::Random;

#[derive(Copy, Clone)]
pub enum AnyEngine {
    Random
}


