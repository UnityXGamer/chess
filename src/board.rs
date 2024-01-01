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

#[derive(Debug, Clone, Copy)]
pub struct Board {
    pub state: State,
    pub pieces: BothColors<PieceBitboards>,
    pub pinned: Bitboard,
    pub check_masks: [Option<(Square, Bitboard, Option<Bitboard>)>; 2],
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

    fn generate_castling_moves<T>(&mut self, color: &Color, mut callback: T)
    where
        T: FnMut(&mut Self, &Move),
    {
        let blockers = self.pieces[color].all | self.pieces[!color].all;
        let mut move_caller = |board: &mut Self,
                               is_kingside: bool,
                               rook_file: File,
                               king_dest_file: File,
                               rook_dest_file: File| {
            let king_sq = Square::from_rank_file(Rank::First.pov(color), File::E);
            let rook_sq = Square::from_rank_file(Rank::First.pov(color), rook_file);
            let rook_dest_sq = Square::from_rank_file(Rank::First.pov(color), rook_dest_file);
            let king_dest_sq = Square::from_rank_file(Rank::First.pov(color), king_dest_file);

            let sqs_between = BETWEEN_LOOKUP[king_sq.idx()][rook_sq.idx()];
            let a1 = board.sq_is_attacked_by(&rook_dest_sq, !color);
            let a2 = board.sq_is_attacked_by(&king_dest_sq, !color);

            if (blockers & sqs_between).is_empty() && !a1 && !a2 {
                callback(
                    board,
                    &Move {
                        piece: Piece::King,
                        from: king_sq,
                        to: king_dest_sq,
                        flag: if is_kingside {
                            MoveFlag::KingSideCastles
                        } else {
                            MoveFlag::QueenSideCastles
                        },
                        promotion: None,
                        old_castling: board.state.castling,
                        old_ep_file: board.state.ep_file,
                    },
                )
            }
        };
        match self.check_masks {
            [None, None] => {
                if self.state.castling[color].king_side {
                    move_caller(self, true, File::H, File::G, File::F);
                }
                if self.state.castling[color].queen_side {
                    move_caller(self, false, File::A, File::C, File::D);
                }
            }
            _ => {}
        }
    }

    pub fn generate_moves<T>(&mut self, mut callback: T)
    where
        T: FnMut(&mut Self, &Move),
    {
        let active_color = &self.state.active_color.clone();
        let opp_color = !active_color;
        let self_king_sq_idx = self.pieces[active_color]
            .king
            .clone()
            .next_sq()
            .expect("king should always exist")
            .idx();

        let king_check_mask = match self.check_masks {
            [Some((_, ray1, _)), Some((_, ray2, _))] => Some(!(ray1 | ray2)),
            [Some((sq, ray, _)), None] => Some(sq.bitboard() | !ray),
            [None, Some((sq, ray, _))] => Some(sq.bitboard() | !ray),
            [None, None] => None,
        };

        let block_check_mask = match self.check_masks {
            // cant block double check, this won't be used anyways
            [Some(_), Some(_)] => None,
            [Some((sq, _, in_between)), None] => {
                Some(sq.bitboard() | in_between.unwrap_or(Bitboard::EMPTY))
            }
            [None, Some((sq, _, in_between))] => {
                Some(sq.bitboard() | in_between.unwrap_or(Bitboard::EMPTY))
            }
            [None, None] => None,
        };

        self.generate_any_moves(
            Piece::King,
            active_color,
            opp_color,
            self_king_sq_idx,
            king_check_mask,
            &mut callback,
        );

        match self.check_masks {
            [Some(_), Some(_)] => {}
            // Only gen other moves if not in double check
            _ => {
                self.generate_castling_moves(active_color, &mut callback);
                self.generate_pawn_moves(
                    active_color,
                    opp_color,
                    self_king_sq_idx,
                    block_check_mask,
                    &mut callback,
                );
                self.generate_any_moves(
                    Piece::Queen,
                    active_color,
                    opp_color,
                    self_king_sq_idx,
                    block_check_mask,
                    &mut callback,
                );
                self.generate_any_moves(
                    Piece::Rook,
                    active_color,
                    opp_color,
                    self_king_sq_idx,
                    block_check_mask,
                    &mut callback,
                );
                self.generate_any_moves(
                    Piece::Bishop,
                    active_color,
                    opp_color,
                    self_king_sq_idx,
                    block_check_mask,
                    &mut callback,
                );
                self.generate_any_moves(
                    Piece::Knight,
                    active_color,
                    opp_color,
                    self_king_sq_idx,
                    block_check_mask,
                    &mut callback,
                );
            }
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

        let get_moves_bb = |from: &Square| match piece {
            Piece::King => Self::get_king_moves(from),
            Piece::Knight => Self::get_knight_moves(from),
            Piece::Bishop => Self::get_bishop_moves(from, opp_all | self_all),
            Piece::Rook => Self::get_rook_moves(from, opp_all | self_all),
            Piece::Queen => Self::get_queen_moves(from, opp_all | self_all),
            Piece::Pawn => unimplemented!("use generate_pawn_moves_instead"),
        };

        for from in piece_bb {
            let mut moves = get_moves_bb(&from) & !self_all;

            if let Some(check_mask) = check_mask {
                moves &= check_mask;
            };

            if piece == Piece::King {
                moves = moves.into_iter().fold(Bitboard::EMPTY, |mut acc, sq| {
                    if !self.sq_is_attacked_by(&sq, opp_color) {
                        acc |= sq.bitboard()
                    }
                    acc
                })
            };

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
        let self_all = self.pieces[active_color].all;
        let pawns = self.pieces[active_color].pawn;

        for from in pawns {
            let mut move_caller = |board: &mut Self, to: Square, flag: MoveFlag| {
                if to.rank() == Rank::Eighth.pov(active_color) {
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

            let pin_mask = if self.pinned.has_sq(from) {
                BETWEEN_RAY_LOOKUP[from.idx()][self_king_sq_idx]
            } else {
                Bitboard::FULL
            };
            let check_mask = if let Some(check_mask) = check_mask {
                check_mask
            } else {
                Bitboard::FULL
            };
            let mask = pin_mask & check_mask;

            // no check mask for now because ep can capture checker without target square being in check mask
            let captures = Self::get_pawn_attacks(&from, active_color) & pin_mask;
            if let Some(ep_file) = self.state.ep_file {
                let to = Square::ep_move_sq(opp_color, ep_file);
                let ep_pawn_sq = Square::ep_pawn_sq(opp_color, ep_file);
                if captures.has_sq(to) && check_mask.has_sq(ep_pawn_sq) {
                    let opp_qr = self.pieces[opp_color].queen | self.pieces[opp_color].rook;
                    let mut can_capture = true;
                    for ep_pinner_sq in BETWEEN_RAY_LOOKUP[self_king_sq_idx][from.idx()] & opp_qr {
                        let mut ib_pieces = BETWEEN_LOOKUP[self_king_sq_idx][ep_pinner_sq.idx()]
                            & (self_all | opp_all);
                        let rel_pieces = (ib_pieces.next(), ib_pieces.next_sq(), ib_pieces.next_sq());
                        
                        if (Some(from), Some(ep_pawn_sq), None) == rel_pieces {
                            can_capture = false;
                            break;
                        } else if (Some(ep_pawn_sq), Some(from), None) == rel_pieces {
                            can_capture = false;
                            break;
                        }
                    }
                    
                    if can_capture {
                        move_caller(self, to, MoveFlag::EnPassant)
                    }
                }
            }
            // add check mask for other captures
            let captures = captures & check_mask & opp_all;
            for to in captures {
                for piece in &Piece::ALL {
                    if self.pieces[opp_color][piece].has_sq(to) {
                        move_caller(self, to, MoveFlag::Capture(*piece))
                    }
                }
            }
            let to = from.increment(match active_color {
                Color::White => 8,
                Color::Black => -8,
            });

            // if to == Square::D6 {
            //     println!("CHECK MASK {} PIN MASK{}", check_mask.unwrap_or(Bitboard::FULL), self.pinned);
            // }

            if (mask & !(self_all | opp_all)).has_sq(to) {
                move_caller(self, to, MoveFlag::None)
            }
            if from.rank() == Rank::Second.pov(active_color) {
                let to = from.increment(match active_color {
                    Color::White => 16,
                    Color::Black => -16,
                });
                if ((BETWEEN_LOOKUP[from.idx()][to.idx()] | to.bitboard()) & (opp_all | self_all))
                    .is_empty()
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

    fn add_checker(&mut self, input: (Square, Bitboard, Option<Bitboard>)) {
        let s = &mut self.check_masks;
        match s {
            [None, None] => s[0] = Some(input),
            [Some(_), None] => s[1] = Some(input),
            _ => unreachable!("self.checkers should not have this conf"),
        }
    }

    pub fn update_slider_checks_pins(&mut self, color: &Color) {
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
        let self_qb = self.pieces[color].queen | self.pieces[color].bishop;
        let self_qr = self.pieces[color].queen | self.pieces[color].rook;

        let rays =
            (self_qr & Self::rook_rays(&opp_king_sq)) | (self_qb & Self::bishop_rays(&opp_king_sq));

        for ray_sq in rays {
            let ray = BETWEEN_RAY_LOOKUP[ray_sq.idx()][opp_king_sq.idx()];
            let in_between = BETWEEN_LOOKUP[ray_sq.idx()][opp_king_sq.idx()];
            let pinned = in_between & (opp_all | self_all);

            match pinned.sq_count() {
                0 => self.add_checker((ray_sq, ray, Some(in_between))),
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

    fn sq_is_attacked_by(&self, sq: &Square, color: &Color) -> bool {
        for p in &Piece::ALL {
            let attacks = self.get_attacks(p, !color, sq);
            let pieces = self.pieces[color][p];
            // println!("SQ {:?} PIECE {:?} PIECES {} ATTAKCS {}", sq, p, pieces, attacks);
            if !(pieces & attacks).is_empty() {
                return true;
            }
        }
        false
    }

    fn toggle_kingside_castles(&mut self, color: &Color) {
        let c = &mut self.state.castling[color].king_side;
        *c = !*c;
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
        let c = &mut self.state.castling[color].queen_side;
        *c = !*c;
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
        self.check_masks = [None, None];
        let active_color = &self.state.active_color.clone();
        let opp_color = !active_color;

        self.state.ep_file = None;
        self.toggle_sq(active_color, &mv.piece, &mv.from);

        match mv.promotion {
            Some(Promotion::Queen) => self.toggle_sq(active_color, &Piece::Queen, &mv.to),
            Some(Promotion::Rook) => self.toggle_sq(active_color, &Piece::Rook, &mv.to),
            Some(Promotion::Bishop) => self.toggle_sq(active_color, &Piece::Bishop, &mv.to),
            Some(Promotion::Knight) => {
                self.toggle_sq(active_color, &Piece::Knight, &mv.to);
                let moves = Self::get_knight_moves(&mv.to);
                if !(moves & self.pieces[opp_color].king).is_empty() {
                    self.add_checker((mv.to, moves, None));
                }
            },
            None => self.toggle_sq(active_color, &mv.piece, &mv.to),
        }

        match &mv.flag {
            MoveFlag::None => {}
            MoveFlag::Capture(piece) => {
                self.toggle_sq(opp_color, piece, &mv.to);
                if *piece == Piece::Rook && mv.to.rank() == Rank::First.pov(opp_color) {
                    let c = &mut self.state.castling[opp_color];
                    match mv.to.file() {
                        File::A => c.queen_side = false,
                        File::H => c.king_side = false,
                        _ => {}
                    }
                }
            }
            MoveFlag::PawnFirstMove => self.state.ep_file = Some(mv.to.file()),
            MoveFlag::EnPassant => self.toggle_sq(
                opp_color,
                &Piece::Pawn,
                &Square::ep_pawn_sq(opp_color, mv.to.file()),
            ),
            MoveFlag::KingSideCastles => self.toggle_kingside_castles(active_color),
            MoveFlag::QueenSideCastles => self.toggle_queenside_castles(active_color),
        }

        match mv.piece {
            Piece::King => {
                let c = &mut self.state.castling[active_color];
                c.king_side = false;
                c.queen_side = false;
            }
            Piece::Rook => {
                let c = &mut self.state.castling[active_color];
                if mv.from.rank() == Rank::First.pov(active_color) {
                    match mv.from.file() {
                        File::A => c.queen_side = false,
                        File::H => c.king_side = false,
                        _ => {}
                    }
                }
            }
            Piece::Knight => {
                let moves = Self::get_knight_moves(&mv.to);
                if !(moves & self.pieces[opp_color].king).is_empty() {
                    self.add_checker((mv.to, moves, None));
                }
            }
            Piece::Pawn => {
                let moves = Self::get_pawn_attacks(&mv.to, active_color);
                if !(moves & self.pieces[opp_color].king).is_empty() {
                    self.add_checker((mv.to, moves, None));
                }
            }

            _ => {}
        }

        self.update_slider_checks_pins(active_color);

        self.state.half_move_count += 1;
        if *active_color == Color::Black {
            self.state.full_move_count += 1;
        }
        self.state.active_color = *opp_color;
    }

    fn unmake_move(&mut self, mv: &Move) {
        self.check_masks = [None, None];
        let opp_color = &self.state.active_color.clone();
        let undoing_color = !opp_color;

        self.toggle_sq(&undoing_color, &mv.piece, &mv.from);

        match mv.promotion {
            Some(Promotion::Queen) => self.toggle_sq(&undoing_color, &Piece::Queen, &mv.to),
            Some(Promotion::Rook) => self.toggle_sq(&undoing_color, &Piece::Rook, &mv.to),
            Some(Promotion::Bishop) => self.toggle_sq(&undoing_color, &Piece::Bishop, &mv.to),
            Some(Promotion::Knight) => self.toggle_sq(&undoing_color, &Piece::Knight, &mv.to),
            None => self.toggle_sq(&undoing_color, &mv.piece, &mv.to),
        }

        match &mv.flag {
            MoveFlag::None => {}
            MoveFlag::Capture(piece) => self.toggle_sq(opp_color, &piece, &mv.to),
            MoveFlag::PawnFirstMove => {}
            MoveFlag::EnPassant => self.toggle_sq(
                opp_color,
                &Piece::Pawn,
                &Square::ep_pawn_sq(opp_color, mv.to.file()),
            ),
            MoveFlag::KingSideCastles => self.toggle_kingside_castles(&undoing_color),
            MoveFlag::QueenSideCastles => self.toggle_queenside_castles(&undoing_color),
        }

        match &mv.piece {
            Piece::King => {
                let moves = Self::get_knight_moves(&mv.from);

                for checker_sq in moves & self.pieces[opp_color].knight {
                    self.add_checker((checker_sq, Self::get_knight_moves(&checker_sq), None));
                }

                let moves = Self::get_pawn_attacks(&mv.from, &undoing_color);

                for checker_sq in moves & self.pieces[opp_color].pawn {
                    self.add_checker((
                        checker_sq,
                        Self::get_pawn_attacks(&mv.from, opp_color),
                        None,
                    ));
                }
            }
            _ => {}
        }

        self.update_slider_checks_pins(opp_color);

        self.state.castling = mv.old_castling;
        self.state.ep_file = mv.old_ep_file;
        self.state.half_move_count -= 1;
        if *undoing_color == Color::Black {
            self.state.full_move_count -= 1;
        }
        self.state.active_color = *undoing_color;
    }

    fn generate_moves_recursively<F>(&mut self, max_ply: u8, current_ply: u8, on_move: &mut F)
    where
        F: FnMut(&mut Board, &Move, u8),
    {
        let callback = |board: &mut Self, mv: &Move| {
            on_move(board, &mv, current_ply);
            if current_ply < max_ply {
                let mut new_board = board.clone();
                new_board.make_move(mv);
                new_board.generate_moves_recursively(max_ply, current_ply + 1, on_move);
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
        let start = std::time::Instant::now();
        let mut on_move = |_: &mut Self, mv: &Move, current_ply: u8| {
            // println!("{}{}", "  ".repeat(current_ply as usize), mv.to_string());
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
        
        let end = start.elapsed();
        
        

        let (to_print, total_count) =
            moves
                .iter()
                .fold((String::new(), 0), |mut acc, (mv, count)| {
                    acc.0 += &format!("{}: {}\n", mv, count);
                    acc.1 += count;
                    acc
                });
        println!("{}\nTotal: {}", to_print, total_count);
        println!("Million moves/sec: {}", 10f64.powf(-6.0) * total_count as f64/end.as_secs_f64());
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
        );
        let checkers = match self.check_masks {
            [Some((sq1, _, _)), Some((sq2, _, _))] => sq1.bitboard() | sq2.bitboard(),
            [Some((sq, _, _)), None] => sq.bitboard(),
            [None, Some((sq, _, _))] => sq.bitboard(),
            [None, None] => Bitboard::EMPTY
        };
        println!(
            "\rCheckers: {}             \r",
            checkers
        );
        println!("\rPinned: {}             \r", self.pinned);
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
        assert_eq!(board.count(5).1, 4865609)
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
        assert_eq!(board.count(5).1, 193690690);
    }

    #[test]
    fn position_3() {
        let mut board =
            Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").expect("test pos 3 is valid");
        assert_eq!(board.count(1).1, 14);
        assert_eq!(board.count(2).1, 191);
        assert_eq!(board.count(3).1, 2812);
        assert_eq!(board.count(4).1, 43238);
        assert_eq!(board.count(5).1, 674624);
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
        assert_eq!(board.count(5).1, 15833292);
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
        assert_eq!(board.count(5).1, 89941194)
    }
}
