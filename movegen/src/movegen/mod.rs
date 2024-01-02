mod king;
mod knight;
mod pawn;
mod slider;
mod util;

// This would be so nice but the rook lookup takes ~45-55 seconds to evaluate to currently sticking to build script
// #[allow(long_running_const_eval)]
// const BISHOP_LOOKUP: [Bitboard; BISHOP_LOOKUP_LEN] = BISHOP.generate_lookup(&BISHOP_MAGIC);
// #[allow(long_running_const_eval)]
// const ROOK_LOOKUP: [Bitboard; ROOK_LOOKUP_LEN] = ROOK.generate_lookup(&ROOK_MAGIC);

pub use king::king_moves;
pub use knight::knight_moves;
pub use pawn::pawn_attacks;
pub use slider::{queen_moves, bishop_moves, bishop_rays, rook_moves, rook_rays};
pub use util::{ray_between, sq_between};