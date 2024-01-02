use util::{color::Color, square::File};

use crate::both_colors::BothColors;

#[derive(Debug, Clone, Copy)]
pub struct State {
    pub active_color: Color,
    pub castling: BothColors<Castling>,
    pub ep_file: Option<File>,
    pub half_move_count: u16,
    pub full_move_count: u16,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Castling {
    pub king_side: bool,
    pub queen_side: bool,
}
