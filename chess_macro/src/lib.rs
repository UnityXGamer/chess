extern crate proc_macro;
use proc_macro::{
    Group, TokenStream,
    TokenTree::{self, Literal},
};

#[proc_macro]
pub fn make_ranks_files_squares(_: TokenStream) -> TokenStream {
    let ranks = (
        "Rank",
        vec![
            "First", "Second", "Third", "Fourth", "Fifth", "Sixth", "Seventh", "Eighth",
        ]
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>(),
    );

    let files = (
        "File",
        vec!["A", "B", "C", "D", "E", "F", "G", "H"]
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>(),
    );
    let squares = (
        "Square",
        ranks
            .1
            .iter()
            .enumerate()
            .flat_map(|(i, _)| files.1.iter().map(move |f| format!("{f}{i}", i = i + 1)))
            .collect::<Vec<String>>(),
    );

    let ranks_code = create_enum_and_impl(ranks.0, ranks.1);
    let files_code = create_enum_and_impl(files.0, files.1);
    let squares_code = create_enum_and_impl(squares.0, squares.1);

    let output = format!("{}\n{}\n{}\n", ranks_code, files_code, squares_code);
    output.parse().expect("Output code is valid")
}

fn create_enum_and_impl(name: &str, variants: Vec<String>) -> String {
    let mut code = format!("#[derive(Debug, Clone, Copy, PartialEq)]\npub enum {name} {{");
    for v in &variants {
        code += &format!("{v},")
    }
    code += "}";

    code += &format!(
        "
        impl {name} {{
            pub const ALL: [Self; {}] = [{}];
            pub const fn from_u8_checked(input: u8) -> Option<Self> {{
                match input {{
                    {}
                    _ => None
                }}
            }}
            pub const fn from_u8(input: u8) -> Self {{
                if let Some(sq) = Self::from_u8_checked(input) {{
                    sq
                }} else {{
                    panic!(\"from_u8 of {name} should be fine\")
                }}
            }}
            pub const fn increment_checked(&self, delta: i8) -> Option<Self> {{
                Self::from_u8_checked((*self as i8+delta) as u8)
            }}
            pub const fn increment(&self, delta: i8) -> Self {{
                Self::from_u8((*self as i8+delta) as u8)
            }}
        }}
    ",
        variants.iter().count(),
        variants.iter().fold(String::new(), |mut acc, v| {
            acc += &format!("Self::{v},");
            acc
        }),
        variants
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (i, v)| {
                acc += &format!("{i} => Some(Self::{v}),");
                acc
            })
    );

    code
}

#[proc_macro]
pub fn make_bitboard(tokens: TokenStream) -> TokenStream {
    let mut bb: u64 = 0;

    let mut tokens = tokens.into_iter();

    for r in (0..8).rev() {
        for f in 0..8 {
            let t = tokens.next().expect("Should have a token for each square");
            match t {
                TokenTree::Ident(l) => {
                    let s = l.to_string();
                    match s.as_str() {
                        "X" => bb |= 1 << (r * 8 + f),
                        s => panic!("Cannot use Ident '{s}' in make_bitboard!"),
                    }
                }
                TokenTree::Punct(p) => {
                    let s = p.to_string();
                    match s.as_str() {
                        "." => {}
                        s => panic!("Cannot use Ident '{s}' in make_bitboard!"),
                    }
                }
                s => panic!("Cannot use TokenTree '{:?}' in make_bitboard!", s),
            }
        }
    }

    format!("Bitboard({})", bb)
        .parse()
        .expect("Output of make_bitboard! should be valid")
}
