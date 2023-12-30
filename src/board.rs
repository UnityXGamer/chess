use std::{collections::HashMap, thread, time::Duration};

include!(concat!(env!("OUT_DIR"), "/lookup.rs"));

use crate::{
    both_colors::BothColors,
    mv::{Move, MoveFlag, Promotion},
    piece_bb::PieceBitboards,
    state::State,
};

use util::{
    bitboard::Bitboard,
    color::Color,
    lookup::{
        generate_between_lookup, generate_between_ray_lookup, generate_king_lookup,
        generate_knight_lookup, generate_pawn_capture_lookup,
    },
    magic::{get_magic_idx, Magic},
    piece::Piece,
    square::{File, Rank, Square},
};

const BETWEEN_LOOKUP: [[Bitboard; 64]; 64] = generate_between_lookup();
const BETWEEN_RAY_LOOKUP: [[Bitboard; 64]; 64] = generate_between_ray_lookup();
const KING_LOOKUP: [Bitboard; 64] = generate_king_lookup();
const KNIGHT_LOOKUP: [Bitboard; 64] = generate_knight_lookup();
const PAWN_CAPTURE_LOOKUP: [[Bitboard; 64]; 2] = generate_pawn_capture_lookup();

// This would be so nice but the rook lookup takes ~45-55 seconds to evaluate to currently sticking to build script
// #[allow(long_running_const_eval)]
// const BISHOP_LOOKUP: [Bitboard; BISHOP_LOOKUP_LEN] = BISHOP.generate_lookup(&BISHOP_MAGIC);
// #[allow(long_running_const_eval)]
// const ROOK_LOOKUP: [Bitboard; ROOK_LOOKUP_LEN] = ROOK.generate_lookup(&ROOK_MAGIC);

#[derive(Debug, Clone)]
pub struct Board {
    pub state: State,
    pub past_states: Vec<State>,
    pub pieces: BothColors<PieceBitboards>,
    pub pinned: Bitboard,
    pub checkers: Bitboard,
}

impl Default for Board {
    fn default() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("default fen is valid")
    }
}

impl Board {
    pub const WIDTH: u8 = 7;

    pub fn get_sq(&self, sq: Square) -> Option<(Color, Piece)> {
        for piece in &Piece::ALL {
            let w = self.pieces[&Color::White][piece];
            let b = self.pieces[&Color::Black][piece];

            if !(w & sq.bitboard()).is_empty() {
                return Some((Color::White, *piece));
            }

            if !(b & sq.bitboard()).is_empty() {
                return Some((Color::Black, *piece));
            }
        }
        None
    }

    pub fn get_sq_moves(&mut self, sq: Square) -> Vec<Move> {
        let mut moves = Vec::new();
        let callback = |_: &mut Self, mv: &Move| {
            if mv.from == sq {
                moves.push(*mv)
            }
        };
        self.generate_moves(callback);
        moves
    }

    pub fn get_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let callback = |_: &mut Self, mv: &Move| {
            moves.push(*mv);
        };
        self.generate_moves(callback);
        moves
    }

    fn xrays(
        magics: &[Magic],
        lookup: &[Bitboard],
        sq: &Square,
        mut blockers: Bitboard,
    ) -> Bitboard {
        let magic = &magics[sq.idx()];
        blockers &= magic.mv_mask;
        let attacks = lookup[get_magic_idx(magic, blockers)];
        blockers & (blockers ^ attacks)
    }

    fn rook_xrays(sq: &Square, blockers: Bitboard) -> Bitboard {
        Self::xrays(&ROOK_MAGIC, &ROOK_LOOKUP, sq, blockers)
    }

    fn bishop_xrays(sq: &Square, blockers: Bitboard) -> Bitboard {
        Self::xrays(&BISHOP_MAGIC, &BISHOP_LOOKUP, sq, blockers)
    }

    fn rays(magics: &[Magic], sq: &Square) -> Bitboard {
        let magic = &magics[sq.idx()];
        magic.mv_mask
    }

    fn rook_rays(sq: &Square) -> Bitboard {
        Self::rays(&ROOK_MAGIC, sq)
    }

    fn bishop_rays(sq: &Square) -> Bitboard {
        Self::rays(&BISHOP_MAGIC, sq)
    }

    fn generate_castling_moves<T>(&mut self, color: &Color, mut callback: T)    where
        T: FnMut(&mut Self, &Move), {
            let blockers = self.pieces[color].all | self.pieces[!color].all;
            let mut move_caller = |board: &mut Self, is_kingside: bool, rook_file: File, king_dest_file: File, rook_dest_file: File|{
                let king_sq = Square::from_rank_file(Rank::First.pov(color), File::E);
                let rook_sq = Square::from_rank_file(Rank::First.pov(color), rook_file);
                let rook_dest_sq = Square::from_rank_file(Rank::First.pov(color), king_dest_file);
                let king_dest_sq = Square::from_rank_file(Rank::First.pov(color), rook_dest_file);

                let sqs_between = BETWEEN_LOOKUP[king_sq.idx()][rook_sq.idx()];
                let a1 = board.sq_is_attacked(color, &rook_dest_sq);
                let a2 = board.sq_is_attacked(color, &king_dest_sq);

                if sqs_between.is_empty() && !a1 && !a1 {
                    callback(board, &Move{
                        piece: Piece::King,
                        from: king_sq,
                        to: king_dest_sq,
                        flag: if is_kingside {
                            MoveFlag::KingSideCastles}
                            else {
                                MoveFlag::QueenSideCastles
                            },
                        promotion: None,
                        old_castling: board.state.castling,
                        old_ep_file: board.state.ep_file,
                    })
                }
            };
            if self.checkers.is_empty() {
                if self.state.castling[color].king_side {
                    move_caller(self, true, File::H, File::G, File::F);
                }
                if self.state.castling[color].queen_side {
                    move_caller(self, true, File::A, File::C, File::D);
                }
            }
        }

    pub fn generate_moves<T>(&mut self, mut callback: T)
    where
        T: FnMut(&mut Self, &Move),
    {
        let active_color = &self.state.active_color.clone();
        let opp_color = !active_color;
        let self_king_sq_idx = self.pieces[active_color].king.clone().next_sq().expect("king should always exist").idx();

        // This assumes only one checker. But check_mask is not used in king movegen anyways and if we are in double check thats the only one that happens.
        let check_mask = if let Some(checker_sq) = self.checkers.clone().next_sq() {
            Some(BETWEEN_LOOKUP[checker_sq.idx()][self_king_sq_idx])
        } else {
            None
        };

        self.generate_any_moves(Piece::King,active_color, opp_color, self_king_sq_idx, check_mask, &mut callback);

        // Only gen other moves if not in double check
        if self.checkers.sq_count() < 2 {
            self.generate_castling_moves(active_color, &mut callback);
            self.generate_pawn_moves(active_color, opp_color, self_king_sq_idx, check_mask, &mut callback);
            self.generate_any_moves(Piece::Queen, active_color, opp_color, self_king_sq_idx, check_mask, &mut callback);
            self.generate_any_moves(Piece::Rook,active_color, opp_color, self_king_sq_idx, check_mask, &mut callback);
            self.generate_any_moves(Piece::Bishop,active_color, opp_color, self_king_sq_idx, check_mask,&mut callback);
            self.generate_any_moves(Piece::Knight,active_color, opp_color, self_king_sq_idx, check_mask,&mut callback);
        }
    }

    fn get_king_moves(from: &Square) -> Bitboard {
        KING_LOOKUP[from.idx()]
    }

    fn get_knight_moves(from: &Square) -> Bitboard {
        KNIGHT_LOOKUP[from.idx()]
    }

    fn get_sliding_moves<const T: usize>(
        from: &Square,
        blockers: Bitboard,
        magics: &[Magic; 64],
        lookup: &[Bitboard; T],
    ) -> Bitboard {
        let magic = magics[from.idx()];
        let moves = lookup[get_magic_idx(&magic, blockers)];
        moves
    }

    fn get_bishop_moves(from: &Square, blockers: Bitboard) -> Bitboard {
        Self::get_sliding_moves(from, blockers, &BISHOP_MAGIC, &BISHOP_LOOKUP)
    }

    fn get_rook_moves(from: &Square, blockers: Bitboard) -> Bitboard {
        Self::get_sliding_moves(from, blockers, &ROOK_MAGIC, &ROOK_LOOKUP)
    }

    fn get_queen_moves(from: &Square, blockers: Bitboard) -> Bitboard {
        Self::get_rook_moves(from, blockers) | Self::get_bishop_moves(from, blockers)
    }

    fn get_pawn_attacks(from: &Square, color: &Color) -> Bitboard {
        PAWN_CAPTURE_LOOKUP[*color as usize][from.idx()]
    }

    fn generate_any_moves<T>(
        &mut self,
        piece: Piece,
        active_color: &Color,
        opp_color: &Color,
        self_king_sq_idx: usize,
        check_mask: Option<Bitboard>,
        mut callback: T,
    ) where
        T: FnMut(&mut Self, &Move),
    {
        let opp_all = self.pieces[opp_color].all;
        let self_all = self.pieces[active_color].all;

        let piece_bb = self.pieces[active_color][&piece];

        let get_moves_bb = |board: &mut Self, from: &Square| match piece {
            Piece::King => Self::get_king_moves(from).into_iter().fold(Bitboard::EMPTY, |mut acc, sq|{if !board.sq_is_attacked(active_color, &sq) {acc|=sq.bitboard()}acc}),
            Piece::Knight => Self::get_knight_moves(from),
            Piece::Bishop => Self::get_bishop_moves(from, opp_all | self_all),
            Piece::Rook => Self::get_rook_moves(from, opp_all | self_all),
            Piece::Queen => Self::get_queen_moves(from, opp_all | self_all),
            Piece::Pawn => unimplemented!("use generate_pawn_moves_instead"),
        };

        for from in piece_bb {
            let mut moves = get_moves_bb(self, &from) & !self_all;

            if piece != Piece::King {
                if let Some(check_mask) = check_mask {
                    moves &= check_mask;
                }
            }

            if self.pinned.has_sq(from) {
                moves &= BETWEEN_RAY_LOOKUP[from.idx()][self_king_sq_idx];
            }

            let mut move_caller = |board: &mut Self, to: Square, flag: MoveFlag| {
                callback(
                    board,
                    &Move {
                        piece,
                        from,
                        to,
                        flag,
                        promotion: None,
                        old_castling: board.state.castling,
                        old_ep_file: board.state.ep_file,
                    },
                );
            };

            let normal_moves = moves & !opp_all;
            for to in normal_moves {
                move_caller(self, to, MoveFlag::None);
            }
            for p in Piece::ALL {
                let captures = moves & self.pieces[opp_color][&p];
                for to in captures {
                    move_caller(self, to, MoveFlag::Capture(p));
                }
            }
        }
    }

    fn generate_pawn_moves<T>(
        &mut self,
        active_color: &Color,
        opp_color: &Color,
        self_king_sq_idx: usize,
        check_mask: Option<Bitboard>,
        mut callback: T,
    ) where
        T: FnMut(&mut Self, &Move),
    {
        let opp_all = self.pieces[opp_color].all;
        let self_all = self.pieces[&active_color].all;
        let pawns = self.pieces[&active_color].pawn;

        for from in pawns {
            let mut move_caller = |board: &mut Self, to: Square, flag: MoveFlag| {
                if to.rank() == Rank::Eighth.pov(&active_color) {
                    for p in Promotion::ALL {
                        callback(
                            board,
                            &Move {
                                piece: Piece::Pawn,
                                from,
                                to,
                                flag,
                                promotion: Some(p),
                                old_castling: board.state.castling,
                                old_ep_file: board.state.ep_file,
                            },
                        );
                    }
                } else {
                    callback(
                        board,
                        &Move {
                            piece: Piece::Pawn,
                            from,
                            to,
                            flag,
                            promotion: None,
                            old_castling: board.state.castling,
                            old_ep_file: board.state.ep_file,
                        },
                    );
                }
            };

            let mask = if self.pinned.has_sq(from) {
                BETWEEN_RAY_LOOKUP[from.idx()][self_king_sq_idx]
            } else {
                Bitboard::FULL
            } & if let Some(check_mask) = check_mask {
                check_mask
            } else {
                Bitboard::FULL
            };

            let captures = Self::get_pawn_attacks(&from, &active_color) & mask;
            if let Some(ep_file) = self.state.ep_file {
                let to = Square::ep_move_sq(&opp_color, ep_file);
                if captures.has_sq(to) {
                    move_caller(self, to, MoveFlag::EnPassant)
                }
            }
            let captures = captures & opp_all;
            for to in captures {
                for piece in &Piece::ALL {
                    if self.pieces[&opp_color][piece].has_sq(to) {
                        move_caller(self, to, MoveFlag::Capture(*piece))
                    }
                }
            }
            let to = from.increment(match active_color {
                Color::White => 8,
                Color::Black => -8,
            });

            if (mask & !(self_all | opp_all)).has_sq(to) {
                move_caller(self, to, MoveFlag::None)
            }
            if from.rank() == Rank::Second.pov(&active_color) {
                let to = from.increment(match active_color {
                    Color::White => 16,
                    Color::Black => -16,
                });
                if (BETWEEN_LOOKUP[from.idx()][to.idx()] & (opp_all | self_all)).is_empty()
                    && mask.has_sq(to)
                {
                    move_caller(self, to, MoveFlag::PawnFirstMove)
                }
            }
        }
    }

    pub fn toggle_sq(&mut self, color: &Color, piece: &Piece, sq: &Square) {
        self.pieces[color][piece] ^= sq.bitboard();
        self.pieces[color].all ^= sq.bitboard();
    }

    pub fn update_slider_checks_pins(&mut self, color: &Color) {
        self.checkers = Bitboard::EMPTY;
        self.pinned = Bitboard::EMPTY;
        let opp_king_sq = self.pieces[&!color]
            .king
            .clone()
            .next_sq()
            .unwrap_or_else(|| {
                self.pretty_print(false);
                panic!("king should always exist");
            });
        let self_all = self.pieces[color].all;
        let opp_all = self.pieces[!color].all;
        let self_qr = self.pieces[color].queen | self.pieces[color].bishop;
        let self_qr = self.pieces[color].queen | self.pieces[color].rook;

        let rays =
            (self_qr & Self::rook_rays(&opp_king_sq)) | (self_qr & Self::bishop_rays(&opp_king_sq));

        for ray_sq in rays {
            let pinned = BETWEEN_LOOKUP[ray_sq.idx()][opp_king_sq.idx()] & (opp_all | self_all);

            match pinned.sq_count() {
                0 => self.checkers |= ray_sq.bitboard(),
                1 => self.pinned |= pinned,
                _ => {}
            }
        }
    }

    // fn update_attacks(&self, color: &Color, mv: &Move) {
    //     let attacks_before = Self::get_attacks(color, mv.from);
    //     let attacks_after = Self::get_attacks(color, mv.to);

    //     for to in attacks_before {
    //         if !self.sq_is_attacked(color, to) {
    //             self.attacks[color] ^= to.bitboard()
    //         }
    //     }

    //     self.attacks[color] |= attacks_after.bitboard();
    // }

    fn get_attacks(&self, piece: &Piece, color: &Color, sq: &Square) -> Bitboard {
        let blockers = self.pieces[color].all | self.pieces[!color].all;
        match piece {
            Piece::Pawn => Self::get_pawn_attacks(sq, color),
            Piece::Knight => Self::get_knight_moves(sq),
            Piece::Bishop => Self::get_bishop_moves(sq, blockers),
            Piece::Rook => Self::get_rook_moves(sq, blockers),
            Piece::Queen => Self::get_queen_moves(sq, blockers),
            Piece::King => Self::get_king_moves(sq),
        }
    }

    fn sq_is_attacked(&self, color: &Color, sq: &Square) -> bool {
        for p in &Piece::ALL {
            if !(self.pieces[!color][p] & self.get_attacks(p, color, sq)).is_empty() {
                return true;
            }
        }
        false
    }

    fn toggle_kingside_castles(&mut self, color: &Color) {
        self.toggle_sq(
            color,
            &Piece::Rook,
            &Square::from_rank_file(Rank::First.pov(color), File::H),
        );
        self.toggle_sq(
            color,
            &Piece::Rook,
            &Square::from_rank_file(Rank::First.pov(color), File::F),
        )
    }

    fn toggle_queenside_castles(&mut self, color: &Color) {
        self.toggle_sq(
            color,
            &Piece::Rook,
            &Square::from_rank_file(Rank::First.pov(color), File::A),
        );
        self.toggle_sq(
            color,
            &Piece::Rook,
            &Square::from_rank_file(Rank::First.pov(color), File::D),
        );
    }

    pub fn make_move(&mut self, mv: &Move) {
        let active_color = self.state.active_color;
        let opp_color = !active_color;

        self.toggle_sq(&active_color, &mv.piece, &mv.from);
        self.toggle_sq(&active_color, &mv.piece, &mv.to);

        match &mv.flag {
            MoveFlag::None => {}
            MoveFlag::Capture(piece) => {
                self.toggle_sq(&opp_color, piece, &mv.to);
                if *piece == Piece::Rook {
                    let c = &mut self.state.castling[&opp_color];
                    match mv.from.file() {
                        File::A => c.queen_side = false,
                        File::H => c.king_side = false,
                        _ => {}
                    }
                }
            }
            MoveFlag::PawnFirstMove => self.state.ep_file = Some(mv.to.file()),
            MoveFlag::EnPassant => self.toggle_sq(
                &opp_color,
                &Piece::Pawn,
                &Square::ep_pawn_sq(&opp_color, mv.to.file()),
            ),
            MoveFlag::KingSideCastles => self.toggle_kingside_castles(&active_color),
            MoveFlag::QueenSideCastles => self.toggle_queenside_castles(&active_color),
        }

        match mv.piece {
            Piece::King => {
                let c = &mut self.state.castling[&active_color];
                c.king_side = false;
                c.queen_side = false;
            }
            Piece::Rook => {
                let c = &mut self.state.castling[&active_color];
                let starting_rank = Rank::First.pov(&active_color);
                if mv.from.rank() == Rank::First.pov(&active_color) {
                    match mv.from.file() {
                        File::A => c.queen_side = false,
                        File::H => c.king_side = false,
                        _ => {}
                    }
                }
            }

            _ => {}
        }

        self.update_slider_checks_pins(&active_color);

        self.state.half_move_count += 1;
        if active_color == Color::Black {
            self.state.full_move_count += 1;
        }
        self.state.active_color = opp_color;
    }

    fn unmake_move(&mut self, mv: &Move) {
        let opp_color = self.state.active_color;
        let undoing_color = !opp_color;

        self.toggle_sq(&undoing_color, &mv.piece, &mv.from);
        self.toggle_sq(&undoing_color, &mv.piece, &mv.to);

        match &mv.flag {
            MoveFlag::None => {}
            MoveFlag::Capture(piece) => self.toggle_sq(&opp_color, &piece, &mv.to),
            MoveFlag::PawnFirstMove => {}
            MoveFlag::EnPassant => self.toggle_sq(
                &opp_color,
                &Piece::Pawn,
                &Square::ep_pawn_sq(&opp_color, mv.to.file()),
            ),
            MoveFlag::KingSideCastles => self.toggle_kingside_castles(&undoing_color),
            MoveFlag::QueenSideCastles => self.toggle_queenside_castles(&undoing_color),
        }

        self.update_slider_checks_pins(&undoing_color);

        self.state.castling = mv.old_castling;
        self.state.ep_file = mv.old_ep_file;
        self.state.half_move_count -= 1;
        if undoing_color == Color::Black {
            self.state.full_move_count -= 1;
        }
        self.state.active_color = undoing_color;
    }

    fn generate_moves_recursively<F>(&mut self, max_ply: u8, current_ply: u8, on_move: &mut F)
    where
        F: FnMut(&mut Board, &Move, u8),
    {
        let callback = |board: &mut Self, mv: &Move| {
            if current_ply < max_ply {
                board.make_move(&mv);
                on_move(board, &mv, current_ply);
                board.generate_moves_recursively(max_ply, current_ply + 1, on_move);
                board.unmake_move(&mv);
            } else {
                board.make_move(&mv);
                on_move(board, &mv, current_ply);
                board.unmake_move(&mv);
            }
        };
        self.generate_moves(callback);
    }

    pub fn play(&mut self, ply: u8, frame_time: u64) {
        let mut on_move = |board: &mut Self, _: &Move, _: u8| {
            board.pretty_print(true);
            thread::sleep(Duration::from_millis(frame_time))
        };
        self.generate_moves_recursively(ply, 1, &mut on_move)
    }

    pub fn count(&mut self, ply: u8) -> (HashMap<String, usize>, usize) {
        let mut moves: HashMap<String, usize> = HashMap::with_capacity(20);
        let mut curr_move: Option<(String, usize)> = None;
        let mut on_move = |_: &mut Self, mv: &Move, current_ply: u8| {
            if let Some((mv, count)) = &mut curr_move {
                if current_ply == 1 {
                    moves.insert(
                        mv.to_string(),
                        match count {
                            0 => 1,
                            c => *c,
                        },
                    );
                } else if current_ply == ply {
                    *count += 1;
                }
            }
            if current_ply == 1 {
                curr_move = Some((mv.to_string(), 0))
            }
        };

        self.generate_moves_recursively(ply, 1, &mut on_move);

        if let Some((mv, count)) = curr_move {
            moves.insert(
                mv.to_string(),
                match count {
                    0 => 1,
                    c => c,
                },
            );
        }

        let (to_print, total_count) =
            moves
                .iter()
                .fold((String::new(), 0), |mut acc, (mv, count)| {
                    acc.0 += &format!("{}: {}\n", mv, count);
                    acc.1 += count;
                    acc
                });
        print!("\n{}\nTotal: {}\n", to_print, total_count);
        (moves, total_count)
    }

    pub fn pretty_print(&self, should_erase: bool) {
        if should_erase {
            print!("{}", "\x1b[1A".repeat(8 * 2 + 3))
        };
        let vertical_line = "---------------------------------";
        println!("\r{}\r", vertical_line);

        for rank in Rank::ALL.iter().rev() {
            let mut to_print = String::from("|");
            for file in File::ALL {
                if let Some((c, p)) = self.get_sq(Square::from_rank_file(*rank, file)) {
                    let mut char = p.to_char();
                    if c == Color::White {
                        char.make_ascii_uppercase()
                    }
                    to_print += &format!(" {char} |",);
                } else {
                    to_print += &format!("   |");
                }
            }
            println!("\r{}\r", to_print);
            println!("\r{}\r", vertical_line);
        }
        println!(
            "\rCurrent turn: {:?}             \r",
            self.state.active_color
        );
        println!(
            "\rEn passant square: {}        \r",
            if let Some(ep_file) = self.state.ep_file {
                Square::ep_move_sq(&self.state.active_color, ep_file).to_string()
            } else {
                "-".to_owned()
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use chess_macro::make_bitboard;
    use util::{bitboard::Bitboard, square::Square};

    use crate::board::{Board, BISHOP_LOOKUP, BISHOP_MAGIC, ROOK_LOOKUP};

    #[test]
    fn moves() {}

    #[test]
    fn xrays1() {
        let blockers = make_bitboard!(
            . . . . X . . .
            . . . X . . . .
            X . X . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
        );
        let expected = make_bitboard!(
            . . . . X . . .
            . . . X . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
        );
        let res = Board::bishop_xrays(&Square::B5, blockers);
        println!("RES {}", res);
        assert_eq!(expected, res);
    }

    #[test]
    fn xrays2() {
        let blockers = make_bitboard!(
            . . X . . . . .
            . . . X . . . .
            . . . . X . . .
            . . . . . . . .
            . . . . X . X .
            . . . X . . . X
            . . X . . . . .
            . X . . . . . .
        );
        let expected = make_bitboard!(
            . . X . . . . .
            . . . X . . . .
            . . . . . . . .
            . . . . . . . .
            . . . . . . . .
            . . . X . . . X
            . . X . . . . .
            . X . . . . . .
        );
        let res = Board::bishop_xrays(&Square::F5, blockers);
        println!("RES {}", res);
        assert_eq!(expected, res);
    }

    #[test]
    fn xrays3() {
        let blockers = make_bitboard!(
            . . . . X . . .
            . . . . . . . .
            . . . . X . . .
            . . . . . . . .
            . . . . X . . .
            . X X X . X . X
            . . . . . . . .
            . . . . . . . .
        );
        let expected = make_bitboard!(
            . . . . X . . .
            . . . . . . . .
            . . . . X . . .
            . . . . . . . .
            . . . . . . . .
            . X X . . . . X
            . . . . . . . .
            . . . . . . . .
        );
        let res = Board::rook_xrays(&Square::E3, blockers);
        println!("RES {}", res);
        assert_eq!(expected, res);
    }

    #[test]
    fn starting_pos() {
        let mut board = Board::default();
        assert_eq!(board.count(1).1, 20);
        assert_eq!(board.count(2).1, 400);
        assert_eq!(board.count(3).1, 8902);
        assert_eq!(board.count(4).1, 197281);
    }

    // See https://www.chessprogramming.org/Perft_Results for positions
    #[test]
    fn kiwipete() {
        let mut board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .expect("kiwipete pos is valid");
        assert_eq!(board.count(1).1, 48);
        assert_eq!(board.count(2).1, 2039);
        assert_eq!(board.count(3).1, 97862);
        assert_eq!(board.count(4).1, 4085603);
    }

    #[test]
    fn position_3() {
        let mut board =
            Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").expect("test pos 3 is valid");
        assert_eq!(board.count(1).1, 14);
        assert_eq!(board.count(2).1, 191);
        assert_eq!(board.count(3).1, 2812);
        assert_eq!(board.count(4).1, 43238);
    }

    #[test]
    fn position_4() {
        let mut board =
            Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .expect("test pos 4 is valid");
        assert_eq!(board.count(1).1, 6);
        assert_eq!(board.count(2).1, 264);
        assert_eq!(board.count(3).1, 9467);
        assert_eq!(board.count(4).1, 422333);
    }

    #[test]
    fn position_5() {
        let mut board =
            Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .expect("test pos 5 is valid");
        assert_eq!(board.count(1).1, 44);
        assert_eq!(board.count(2).1, 1486);
        assert_eq!(board.count(3).1, 62379);
        assert_eq!(board.count(4).1, 2103487);
    }
}
