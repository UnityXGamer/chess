use util::{magic::ROOK, precomputed::ROOK_MAGIC};

fn main() {
    let _ = ROOK.generate_lookup(&ROOK_MAGIC);
}
