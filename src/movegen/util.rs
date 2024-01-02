use util::{bitboard::Bitboard, square::Square, magic::DELTAS};

const fn generate_sq_between_lookup() -> [[Bitboard; 64]; 64] {
    let mut lookup: [[Bitboard; 64]; 64] = [[Bitboard::EMPTY; 64]; 64];
    let mut i = 0;
    while i < Square::ALL.len() {
        let outer_sq = Square::from_u8(i as u8);
        let mut d = 0;
        while d <= 7 {
            let mut inner_sq = outer_sq;
            let mut curr_in_between = Bitboard::EMPTY;
            while let Some(sq) = inner_sq.apply_delta(DELTAS[d]) {
                lookup[i][sq.idx()] = curr_in_between;
                curr_in_between.0 |= sq.bitboard().0;
                inner_sq = sq;
            }
            d += 1;
        }
        i += 1;
    }
    lookup
}

const SQ_BETWEEN: [[Bitboard; 64]; 64] = generate_sq_between_lookup();

pub fn sq_between(sq1: &Square, sq2: &Square) -> Bitboard {
    SQ_BETWEEN[sq1.idx()][sq2.idx()]
}

const fn generate_ray_between_lookup() -> [[Bitboard; 64]; 64] {
    let mut lookup: [[Bitboard; 64]; 64] = [[Bitboard::EMPTY; 64]; 64];
    let mut i = 0;
    while i < Square::ALL.len() {
        let outer_sq = Square::from_u8(i as u8);
        let mut d = 0;
        while d <= 7 {
            let mut inner_sq = outer_sq;
            let mut ray = Bitboard::EMPTY;

            while let Some(sq) = inner_sq.apply_delta(DELTAS[d]) {
                ray.0 |= sq.bitboard().0;
                inner_sq = sq;
            }
            
            let (d_rank, d_file) = DELTAS[d];
            let d_neg = (-d_rank, -d_file);
            
            while let Some(sq) = inner_sq.apply_delta(d_neg) {
                ray.0 |= sq.bitboard().0;
                inner_sq = sq;
            }

            inner_sq = outer_sq;
            while let Some(sq) = inner_sq.apply_delta(DELTAS[d]) {
                lookup[i][sq.idx()] = ray;
                inner_sq = sq;
            }

            d += 1;
        }
        i += 1;
    }
    lookup
}

const RAY_BETWEEN: [[Bitboard; 64]; 64] = generate_ray_between_lookup();

pub fn ray_between(sq1: &Square, sq2: &Square) -> Bitboard {
    RAY_BETWEEN[sq1.idx()][sq2.idx()]
}

