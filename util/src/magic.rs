use std::time::{SystemTime, UNIX_EPOCH};

use crate::{bitboard::Bitboard, square::Square};

#[derive(Debug, Clone, Copy)]
pub struct Magic {
    /// This is the mask used in the magic lookup.
    /// This does not include the last square in each direction the piece can attack.
    /// The last squares in each direction don't matter, because whether they contain a piece or not, they will be added to the moves anyways and friendly captures etc. will be filtered out.
    pub mask: Bitboard,
    /// This is the bitboard that contains all the possible moves the piece can make from the given square without any blockers
    pub mv_mask: Bitboard,
    pub magic: u64,
    pub offset: usize,
    pub shift: u8,
}

impl Magic {
    pub const fn placeholder() -> Self {
        Self {
            mask: Bitboard::EMPTY,
            mv_mask: Bitboard::EMPTY,
            magic: 0,
            offset: 0,
            shift: 0,
        }
    }
}
/// Blockers are masked in this function
/// Just pass in `opp_all | self_all` or equivalent for blockers
pub const fn get_magic_idx(magic: &Magic, blockers: Bitboard) -> usize {
    let idx =
        ((blockers.0 & magic.mask.0).wrapping_mul(magic.magic) >> magic.shift as usize) as usize;
    idx + magic.offset
}

pub const DELTAS: [(i8, i8); 8] = [
    (-1, 0),
    (0, -1),
    (1, 0),
    (0, 1),
    (-1, -1),
    (-1, 1),
    (1, -1),
    (1, 1),
];
 const ORTHOGONALS: [(i8, i8); 4] = [DELTAS[0], DELTAS[1], DELTAS[2], DELTAS[3]];

 const DIAGONALS: [(i8, i8); 4] = [DELTAS[4], DELTAS[5], DELTAS[6], DELTAS[7]];

 pub struct Rand(u64, u64);

 impl Rand {
     fn new() -> Self {
         let t = SystemTime::now().duration_since(UNIX_EPOCH).expect("systemtime is fine").subsec_nanos() as u64;
         Self(t, t)
     }
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

pub struct MagicPiece<const LOOKUP_LEN: usize> {
    deltas: [(i8, i8); 4],
    piece_name: &'static str,
}

impl<const LOOKUP_LEN: usize> MagicPiece<LOOKUP_LEN> {
    pub const fn generate_lookup(
        &self,
        precomputed_magics: &[Magic; 64],
    ) -> [Bitboard; LOOKUP_LEN] {
        let mut lookup = [Bitboard::EMPTY; LOOKUP_LEN];
        let mut i = 0;

        while i < Square::ALL.len() {
            let sq = Square::from_u8(i as u8);
            let magic = precomputed_magics[i as usize];
            let mut subset = Bitboard::EMPTY;
            // see https://www.chessprogramming.org/Traversing_Subsets_of_a_Set#All_Subsets_of_any_Set
            loop {
                let idx = get_magic_idx(&magic, subset);
                if lookup[idx].is_empty() {
                    lookup[idx] = self.moves(&sq, &subset);
                } else {
                    panic!("Lookup overlap")
                }
                subset.0 = subset.0.wrapping_sub(magic.mask.0) & magic.mask.0;
                if subset.0 == 0 {
                    break;
                }
            }
            i += 1;
        }
        lookup
    }
    pub fn get_magic_lookup_code(
        &self,
        precomputed_magics: Option<&[Magic; 64]>,
        include_lookup_table: bool,
    ) -> String {
        let mut magic_code = format!("\npub const {}_MAGIC: [Magic; 64] = [\n", self.piece_name);
        let mut lookup_code = String::new();
        let mut curr_offset = 0;
        let mut rand = Rand::new();
        for sq in Square::ALL {
            let (lookup, magic) = match precomputed_magics {
                Some(magics) => {
                    let magic = magics[sq.idx()];
                    let subsets = magic
                        .mask
                        .subsets()
                        .iter()
                        .map(|s| (*s, self.moves(&sq, s)))
                        .collect::<Vec<(Bitboard, Bitboard)>>();
                    (
                        self.get_lookup_for_magic(&subsets, &magic)
                            .expect("precomputed magic is valid"),
                        magic,
                    )
                }
                None => self.generate_magic(&sq, curr_offset, &mut rand),
            };
            curr_offset += lookup.len();
            magic_code += &format!(
                "  Magic {{ mask: Bitboard({mask}), mv_mask: Bitboard({mv_mask}), magic: {magic}, offset: {offset}, shift: {shift}}},\n",
                mask = magic.mask.0,
                mv_mask = magic.mv_mask.0,
                offset = magic.offset,
                magic = magic.magic,
                shift = magic.shift
            );
            if include_lookup_table {
                lookup_code += &format!(
                    "{}\n",
                    lookup
                        .iter()
                        .map(|val| format!("Bitboard({}),", val.0))
                        .fold(String::new(), |mut acc, val| {
                            acc += &val;
                            acc
                        })
                );
            }
        }
        magic_code += "];";
        let lookup_code = format!(
            "\npub const {}_LOOKUP: [Bitboard; {}] = [\n{}];",
            self.piece_name, curr_offset, lookup_code
        );
        if include_lookup_table {
            magic_code += &lookup_code
        }
        magic_code
    }
    pub fn mask(&self, start_sq: &Square) -> Bitboard {
        let mut bitboard = Bitboard::EMPTY;
        for delta in self.deltas {
            let mut curr_sq = *start_sq;
            while let Some(sq) = curr_sq.apply_delta(delta) {
                bitboard |= curr_sq.bitboard();
                curr_sq = sq;
            }
        }
        bitboard &= !start_sq.bitboard();
        bitboard
    }
    pub const fn moves(&self, start_sq: &Square, blockers: &Bitboard) -> Bitboard {
        let mut bitboard = Bitboard::EMPTY;
        let mut i = 0;
        while i < self.deltas.len() {
            let mut curr_sq = *start_sq;
            while let Some(sq) = curr_sq.apply_delta(self.deltas[i]) {
                bitboard.0 |= sq.bitboard().0;
                if blockers.0 & sq.bitboard().0 != 0 {
                    break;
                }
                curr_sq = sq;
            }
            i += 1;
        }
        bitboard.0 &= !start_sq.bitboard().0;
        bitboard
    }

    fn get_lookup_for_magic(
        &self,
        subsets: &Vec<(Bitboard, Bitboard)>,
        magic: &Magic,
    ) -> Option<Vec<Bitboard>> {
        let mut lookup: Vec<Bitboard> = vec![Bitboard::EMPTY; 1 << (64 - magic.shift)];
        for (s, moves) in subsets {
            let bb = &mut lookup[get_magic_idx(&magic, *s) - magic.offset];
            if bb.is_empty() {
                *bb = *moves;
            } else {
                return None;
            }
        }
        Some(lookup)
    }

    pub fn generate_magic(
        &self,
        sq: &Square,
        offset: usize,
        rand: &mut Rand,
    ) -> (Vec<Bitboard>, Magic) {
        let mask = self.mask(&sq);
        let mv_mask = self.moves(&sq, &Bitboard::EMPTY);
        let lookup_bits = mask.0.count_ones();
        let shift = 64 - lookup_bits as u8;
        let subsets = mask
            .subsets()
            .iter()
            .map(|s| (*s, self.moves(sq, s)))
            .collect::<Vec<(Bitboard, Bitboard)>>();
        loop {
            let (r1, r2, r3) = (rand.next(), rand.next(), rand.next());
            let magic = r1 & r2 & r3;
            let magic = Magic {
                mv_mask,
                mask,
                magic,
                offset,
                shift,
            };
            if let Some(lookup) = self.get_lookup_for_magic(&subsets, &magic) {
                println!("MAGIC GENERATED FOR SQ {:?}", sq);
                return (lookup, magic);
            }
        }
    }
}

pub const ROOK_LOOKUP_LEN: usize = 102400;
pub const ROOK: MagicPiece<ROOK_LOOKUP_LEN> = MagicPiece {
    deltas: ORTHOGONALS,
    piece_name: "ROOK",
};

pub const BISHOP_LOOKUP_LEN: usize = 5248;
pub const BISHOP: MagicPiece<BISHOP_LOOKUP_LEN> = MagicPiece {
    deltas: DIAGONALS,
    piece_name: "BISHOP",
};
