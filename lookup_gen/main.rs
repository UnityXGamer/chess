use std::{env, fs, path::Path};
use util::{
    lookup::{between_lookup_code, king_lookup_code, knight_lookup_code},
    magic::{BISHOP, ROOK},
    precomputed::{BISHOP_MAGIC, ROOK_MAGIC},
};

fn main() {
    let out_dir = env::var_os("OUT_DIR").expect("reading env succeeds");
    let dest_path = Path::new(&out_dir).join("lookup.rs");

    let bishop_code = BISHOP.get_magic_lookup_code(Some(&BISHOP_MAGIC), true);
    let rook_code = ROOK.get_magic_lookup_code(Some(&ROOK_MAGIC), true);

    fs::write(dest_path, format!("{}{}", bishop_code, rook_code))
        .expect("writing lookups succeeds");
}
