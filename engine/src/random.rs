use std::{thread, time::Duration};

use movegen::board::{Board, Status};

use crate::{Engine, MoveSearch};

#[derive(Debug, Clone, Copy)]
pub struct Rand(u64, u64);

impl Rand {
    /// see https://en.wikipedia.org/wiki/Xorshift#xorshift+
    fn next(&mut self) -> u64 {
        let mut t = self.0;
        let s = self.1;
        self.0 = s;
        t ^= t << 23;
        t ^= t >> 18;
        t ^= s ^ (s >> 5);
        self.1 = t;
        return t.wrapping_add(s);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Random {
    rand: Rand
}
impl MoveSearch for Random {
    type Input = u64;
    fn init(rand_seed: Self::Input) -> Self {
        Self {
            rand: Rand(rand_seed, rand_seed)
        }
    }
    fn search(&mut self, board: &mut Board, best_move: &mut Option<movegen::mv::Move>, ms_remaining: u64) {
        match board.status() {
            Status::Ongoing(moves) => {
                let idx = self.rand.next() as usize % moves.len();
                *best_move = Some(moves[idx]);
            }
            _ => {}
        }
    }
}