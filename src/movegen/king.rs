use util::{square:: Square, bitboard::Bitboard, magic::DELTAS};

const fn generate_king_lookup() -> [Bitboard; 64] {
    let mut lookup = [Bitboard::EMPTY; 64];
    let mut i = 0;
    while i < Square::ALL.len() {
        let sq = Square::from_u8(i as u8);
        let mut bb = Bitboard::EMPTY;
        let mut d = 0;
        while d <= 7 {
            if let Some(sq) = sq.apply_delta(DELTAS[d]) {
                bb.0 |= sq.bitboard().0
            }
            d += 1;
        }
        lookup[i] = bb;
        i += 1;
    }
    lookup
}

const KING_LOOKUP: [Bitboard; 64] = generate_king_lookup();

pub fn king_moves(sq: &Square) -> Bitboard {
    KING_LOOKUP[sq.idx()]
}