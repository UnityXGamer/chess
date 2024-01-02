include!(concat!(env!("OUT_DIR"), "/lookup.rs"));

use util::{bitboard::Bitboard, magic::{get_magic_idx, Magic}, square::Square};

fn sliding_moves<const T: usize>(
    from: &Square,
    blockers: Bitboard,
    magics: &[Magic; 64],
    lookup: &[Bitboard; T],
) -> Bitboard {
    let magic = magics[from.idx()];
    let moves = lookup[get_magic_idx(&magic, blockers)];
    moves
}

pub fn bishop_moves(from: &Square, blockers: Bitboard) -> Bitboard {
    sliding_moves(from, blockers, &BISHOP_MAGIC, &BISHOP_LOOKUP)
}

pub fn rook_moves(from: &Square, blockers: Bitboard) -> Bitboard {
    sliding_moves(from, blockers, &ROOK_MAGIC, &ROOK_LOOKUP)
}

pub fn queen_moves(from: &Square, blockers: Bitboard) -> Bitboard {
    rook_moves(from, blockers) | bishop_moves(from, blockers)
}

fn rays(magics: &[Magic], sq: &Square) -> Bitboard {
    let magic = &magics[sq.idx()];
    magic.mv_mask
}

pub fn rook_rays(sq: &Square) -> Bitboard {
    rays(&ROOK_MAGIC, sq)
}

pub fn bishop_rays(sq: &Square) -> Bitboard {
    rays(&BISHOP_MAGIC, sq)
}
