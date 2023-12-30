use crate::{
    bitboard::Bitboard,
    color::Color,
    magic::DELTAS,
    square::{File, Rank, Square},
};

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

pub fn king_lookup_code() -> String {
    format!(
        "pub const KING_LOOKUP: [Bitboard; 64] = {};\n",
        array_code(generate_king_lookup())
    )
}

pub fn knight_lookup_code() -> String {
    format!(
        "pub const KNIGHT_LOOKUP: [Bitboard; 64] = {};\n",
        array_code(generate_knight_lookup())
    )
}

pub fn between_lookup_code() -> String {
    format!(
        "pub const BETWEEN_LOOKUP: [[Bitboard; 64]; 64] = [\n{}];\n",
        generate_between_lookup()
            .iter()
            .fold(String::new(), |mut acc, bbs| {
                acc += &format!("{},\n", array_code(*bbs));
                acc
            })
    )
}

fn array_code(arr: [Bitboard; 64]) -> String {
    format!(
        "[{}]",
        arr.iter().fold(String::new(), |mut acc, bb| {
            acc += &format!("Bitboard({}),", bb.0);
            acc
        })
    )
}

pub const fn generate_between_lookup() -> [[Bitboard; 64]; 64] {
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

pub const fn generate_between_ray_lookup() -> [[Bitboard; 64]; 64] {
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

pub const fn generate_king_lookup() -> [Bitboard; 64] {
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

pub const fn generate_pawn_capture_lookup() -> [[Bitboard; 64]; 2] {
    let mut lookup = [[Bitboard::EMPTY; 64]; 2];
    let mut i = 0;
    while i < Square::ALL.len() {
        let sq = Square::from_u8(i as u8);
        let sq_bb = sq.bitboard().0;
        let rank = sq.rank();
        let file = sq.file();

        match (rank, file) {
            (Rank::First, _) => {}
            (Rank::Eighth, _) => {}
            (_, file) => {
                // Captures to the right from white's pov
                if file as u8 != File::H as u8 {
                    lookup[Color::White as usize][i].0 |= sq_bb << 9;
                    lookup[Color::Black as usize][i].0 |= sq_bb >> 7;
                }
                // Captures to the left from white's pov
                if file as u8 != File::A as u8 {
                    lookup[Color::White as usize][i].0 |= sq_bb << 7;
                    lookup[Color::Black as usize][i].0 |= sq_bb >> 9;
                }
            }
        }
        i += 1;
    }
    lookup
}

pub const fn generate_knight_lookup() -> [Bitboard; 64] {
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
