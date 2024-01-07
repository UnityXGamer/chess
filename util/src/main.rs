use util::{magic::ROOK, precomputed::ROOK_MAGIC};

fn main() {
    println!("{}",ROOK.get_magic_lookup_code(None, false));
}
