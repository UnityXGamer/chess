use util::{square::{Rank, File, Square}, bitboard::Bitboard, color::Color};

const fn generate_pawn_capture_lookup() -> [[Bitboard; 64]; 2] {
    const COLORS: [Color; 2] = [Color::White, Color::Black];
    let mut lookup = [[Bitboard::EMPTY; 64]; 2];

    let mut c = 0;
    while c < COLORS.len() {
        let color = COLORS[c];
        let mut i = 0;
        while i < Square::ALL.len() {
            let sq = Square::from_u8(i as u8);
            let sq_bb = sq.bitboard().0;
            let rank = sq.rank();
            let file = sq.file();
    
            if rank as u8 != Rank::Eighth.pov(&color) as u8 {
                // Captures to the right from white's pov
                if file as u8 != File::H as u8 {
                    lookup[color as usize][i].0 |= match color {
                        Color::White => sq_bb << 9,
                        Color::Black =>sq_bb >> 7,
                    }
                }
                // Captures to the left from white's pov
                if file as u8 != File::A as u8 {
                    lookup[color as usize][i].0 |= match color {
                        Color::White => sq_bb << 7,
                        Color::Black =>sq_bb >> 9,
                    }
                }
            }
            i += 1;
        }
        c+=1;
    }

    lookup
}

const PAWN_CAPTURE_LOOKUP: [[Bitboard; 64]; 2] = generate_pawn_capture_lookup();

pub fn pawn_attacks(color: Color, sq: &Square) -> Bitboard {
    PAWN_CAPTURE_LOOKUP[color as usize][sq.idx()]
}