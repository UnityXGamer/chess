use util::{bitboard::Bitboard, square::Square};

const KNIGHT_JUMPS: [(i8, i8); 8] = [
    (-2, -1),
    (-1, -2),
    (-2, 1),
    (-1, 2),
    (1, -2),
    (2, -1),
    (1, 2),
    (2, 1),
];

const fn generate_knight_lookup() -> [Bitboard; 64] {
    let mut lookup = [Bitboard::EMPTY; 64];
    let mut i = 0;
    while i < Square::ALL.len() {
        let sq = Square::from_u8(i as u8);
        let mut bb = Bitboard::EMPTY;
        let mut d = 0;
        while d <= 7 {
            if let Some(sq) = sq.apply_delta(KNIGHT_JUMPS[d]) {
                bb.0 |= sq.bitboard().0
            }
            d += 1;
        }
        lookup[i] = bb;
        i += 1;
    }
    lookup
}

const KNIGHT_LOOKUP: [Bitboard; 64] = generate_knight_lookup();

pub fn knight_moves(sq: &Square) -> Bitboard {
    KNIGHT_LOOKUP[sq.idx()]
}