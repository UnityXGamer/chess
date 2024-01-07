use std::{thread, time::Duration};

use crate::{
    both_colors::BothColors,
    mv::{Move, MoveFlag, Promotion},
    piece_bb::PieceBitboards,
    state::State,
    movegen::{
        pawn_attacks,
        knight_moves,
        bishop_moves,
        bishop_rays,
        rook_moves,
        rook_rays,
        queen_moves,
        sq_between,
        ray_between, king_moves
    }
};

use util::{
    bitboard::Bitboard,
    color::Color,
    piece::Piece,
    square::{File, Square, Rank},
};

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

#[derive(Debug)]
pub enum Status {
    Checkmate,
    Stalemate,
    Draw,
    Ongoing(Vec<Move>)
}


impl Board {
    pub const WIDTH: u8 = 7;
    
    pub fn in_check(&self) -> bool {
        match self.check_masks {
            [None, None] => false,
            _ => true
        }
    }
    
    pub fn status(&mut self) -> Status {
        let moves = self.get_moves();
        if moves.len() == 0 {
            if self.in_check() {
                Status::Checkmate
            } else {
                Status::Stalemate
            }
        } else {
            Status::Ongoing(moves)
        }
    }
    
    pub fn all_sqs(&self, white_pov: bool) -> Vec<Vec<(Square, Option<(Color, Piece)>)>> {
        let mut output = Vec::with_capacity(8);
        
        let iter = Rank::ALL.iter();
        
        let rank_callback = |rank: &Rank|{
            output.push(Vec::with_capacity(8));
            File::ALL.iter().for_each(|file|{
                let sq = Square::from_rank_file(*rank, *file);
                output.last_mut().expect("always has row").push((sq, self.get_sq(sq)))
            })
        };
        
        if white_pov {
            iter.rev().for_each(rank_callback)
        } else {
            iter.for_each(rank_callback)
        }
        
        output
    }

    fn get_sq(&self, sq: Square) -> Option<(Color, Piece)> {
        for piece in &Piece::ALL {
            let w = self.pieces[&Color::White][piece];
            let b = self.pieces[&Color::Black][piece];

            if w.has_sq(sq) {
                return Some((Color::White, *piece));
            }

            if b.has_sq(sq) {
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

            let sqs_between = sq_between(&king_sq, &rook_sq);
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
                        promotion: None
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
        let self_king_sq = self.pieces[active_color]
            .king
            .clone()
            .next_sq()
            .expect("king should always exist");

        let king_check_mask = match self.check_masks {
            // Allow king to capture a checker if it is not in other checker's ray
            // Checking if the piece is protected by a non checking piece happens later
            [Some((sq1, ray1, _)), Some((sq2, ray2, _))] => Some((sq1.bitboard() & !ray2) | (sq2.bitboard() & !ray1) | !(ray1 | ray2)),
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
            &self_king_sq,
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
                    &self_king_sq,
                    block_check_mask,
                    &mut callback,
                );
                self.generate_any_moves(
                    Piece::Queen,
                    active_color,
                    opp_color,
                    &self_king_sq,
                    block_check_mask,
                    &mut callback,
                );
                self.generate_any_moves(
                    Piece::Rook,
                    active_color,
                    opp_color,
                    &self_king_sq,
                    block_check_mask,
                    &mut callback,
                );
                self.generate_any_moves(
                    Piece::Bishop,
                    active_color,
                    opp_color,
                    &self_king_sq,
                    block_check_mask,
                    &mut callback,
                );
                self.generate_any_moves(
                    Piece::Knight,
                    active_color,
                    opp_color,
                    &self_king_sq,
                    block_check_mask,
                    &mut callback,
                );
            }
        }
    }

    fn generate_any_moves<T>(
        &mut self,
        piece: Piece,
        active_color: &Color,
        opp_color: &Color,
        self_king_sq: &Square,
        check_mask: Option<Bitboard>,
        mut callback: T,
    ) where
        T: FnMut(&mut Self, &Move),
    {
        let opp_all = self.pieces[opp_color].all;
        let self_all = self.pieces[active_color].all;

        let piece_bb = self.pieces[active_color][&piece];

        for from in piece_bb {
            let mut moves = self.get_attacks(&piece, active_color, &from) & !self_all;

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
                moves &= ray_between(&from, self_king_sq);
            }
            let mut move_caller = |board: &mut Self, to: Square, flag: MoveFlag| {
                callback(
                    board,
                    &Move {
                        piece,
                        from,
                        to,
                        flag,
                        promotion: None
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
        self_king_sq: &Square,
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
                                promotion: Some(p)
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
                            promotion: None
                        },
                    );
                }
            };

            let pin_mask = if self.pinned.has_sq(from) {
                ray_between(&from, &self_king_sq)
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
            let captures = pawn_attacks(*active_color, &from) & pin_mask;
            if let Some(ep_file) = self.state.ep_file {
                let to = Square::ep_move_sq(opp_color, ep_file);
                let ep_pawn_sq = Square::ep_pawn_sq(opp_color, ep_file);
                if captures.has_sq(to) && check_mask.has_sq(ep_pawn_sq) {
                    let opp_qr = self.pieces[opp_color].queen | self.pieces[opp_color].rook;
                    let mut can_capture = true;
                    for ep_pinner_sq in ray_between(self_king_sq, &from) & opp_qr {
                        let mut ib_pieces = sq_between(self_king_sq, &ep_pinner_sq)
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
                if ((sq_between(&from, &to) | to.bitboard()) & (opp_all | self_all))
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
            (self_qr & rook_rays(&opp_king_sq)) | (self_qb & bishop_rays(&opp_king_sq));

        for ray_sq in rays {
            let ray = ray_between(&ray_sq, &opp_king_sq);
            let in_between = sq_between(&ray_sq, &opp_king_sq);
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
            Piece::Pawn => pawn_attacks(*color, sq),
            Piece::Knight => knight_moves(sq),
            Piece::Bishop => bishop_moves(sq, blockers),
            Piece::Rook => rook_moves(sq, blockers),
            Piece::Queen => queen_moves(sq, blockers),
            Piece::King => king_moves(sq),
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
                let moves = knight_moves(&mv.to);
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
                let moves = knight_moves(&mv.to);
                if !(moves & self.pieces[opp_color].king).is_empty() {
                    self.add_checker((mv.to, moves, None));
                }
            }
            Piece::Pawn => {
                let moves = pawn_attacks(*active_color, &mv.to);
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
    
    pub fn perft_multithread(&mut self, ply: u8, thread_count: usize) -> usize {
        let mut moves = Vec::with_capacity(30);
        let mut total = 0;
        let start = std::time::Instant::now();
        let get_first_moves = |_: &mut Self, mv: &Move| {
            moves.push(*mv);
        };
        
        self.generate_moves(get_first_moves);
        
        for moves in moves.chunks(thread_count) {
            let mut threads = Vec::with_capacity(thread_count);
            for mv in moves {
                let mut board = self.clone();
                let mv = *mv;
                threads.push((mv, thread::spawn(move||{
                    let mut mv_total = 0;
                    board.make_move(&mv);
                    if ply >= 2 {
                        let mut on_move = |_: &mut Self, _: &Move, current_ply: u8| {
                            if &current_ply == &ply {
                                mv_total+=1;
                            }
                        };
                        board.generate_moves_recursively(ply, 2, &mut on_move)
                    }
                    
                    match mv_total {
                        0 if ply == 1 => 1,
                        t => t
                    }
                })));
            }
            for (mv, t) in threads {
                let mv_total=t.join().expect("joining thread is fine");
                println!("{}: {}", mv.to_string(), mv_total);
                total+=mv_total;
            }
        }
        let end = start.elapsed();
        let m_moves_per_sec = 10f64.powf(-6.0) * total as f64/end.as_secs_f64();
        println!("Total: {}", total);
        println!("Million moves/sec: {}", m_moves_per_sec);
        println!("Million moves/sec/thread ({thread_count} threads): {}", m_moves_per_sec/thread_count as f64);
        total
    }

    pub fn perft(&mut self, ply: u8) -> usize {
        let mut curr_move: Option<(String, usize)> = None;
        let mut total = 0;
        let start = std::time::Instant::now();
        let mut on_move = |_: &mut Self, mv: &Move, current_ply: u8| {
            // println!("{}{}", "  ".repeat(current_ply as usize), mv.to_string());
            if let Some((mv, count)) = &mut curr_move {
                if current_ply == 1 {
                    let count = match count {
                        0 => 1,
                        c => *c,
                    };
                    println!("{}: {}", mv, count);
                    total+=count;
                    curr_move = None;
                } else if current_ply == ply {
                    *count += 1;
                }
            } 
            if curr_move.is_none() {
                curr_move = Some((mv.to_string(), 0))
            }
        };

        self.generate_moves_recursively(ply, 1, &mut on_move);

        if let Some((mv, count)) = curr_move {
            let count = match count {
                0 => 1,
                c => c,
            };
            println!("{}: {}", mv, count);
            total+=count;
        }
        
        let end = start.elapsed();
        println!("Total: {}", total);
        println!("Million moves/sec: {}", 10f64.powf(-6.0) * total as f64/end.as_secs_f64());
        total
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
    use crate::board::Board;

    #[test]
    fn starting_pos() {
        let mut board = Board::default();
        assert_eq!(board.perft_multithread(1, 8), 20);
        assert_eq!(board.perft_multithread(2, 8), 400);
        assert_eq!(board.perft_multithread(3, 8), 8_902);
        assert_eq!(board.perft_multithread(4, 8), 197_281);
        assert_eq!(board.perft_multithread(5, 8), 4_865_609);
        assert_eq!(board.perft_multithread(6, 8), 119_060_324);
        assert_eq!(board.perft_multithread(7, 8), 3_195_901_860);
    }

    // See https://www.chessprogramming.org/Perft_Results for positions
    #[test]
    fn kiwipete() {
        let mut board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .expect("kiwipete pos is valid");
        assert_eq!(board.perft_multithread(1, 8), 48);
        assert_eq!(board.perft_multithread(2, 8), 2_039);
        assert_eq!(board.perft_multithread(3, 8), 97_862);
        assert_eq!(board.perft_multithread(4, 8), 4_085_603);
        assert_eq!(board.perft_multithread(5, 8), 193_690_690);
        assert_eq!(board.perft_multithread(6, 8), 8_031_647_685);
    }


    #[test]
    fn position_3() {
        let mut board =
            Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").expect("test pos 3 is valid");
        assert_eq!(board.perft_multithread(1, 8), 14);
        assert_eq!(board.perft_multithread(2, 8), 191);
        assert_eq!(board.perft_multithread(3, 8), 2_812);
        assert_eq!(board.perft_multithread(4, 8), 43_238);
        assert_eq!(board.perft_multithread(5, 8), 674_624);
        assert_eq!(board.perft_multithread(6, 8), 11_030_083);
        assert_eq!(board.perft_multithread(7, 8), 178_633_661);
        // assert_eq!(board.perft_multithread(8, 8), 3_009_794_393);
    }

    #[test]
    fn position_4() {
        let mut board =
            Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .expect("test pos 4 is valid");
        assert_eq!(board.perft_multithread(1, 8), 6);
        assert_eq!(board.perft_multithread(2, 8), 264);
        assert_eq!(board.perft_multithread(3, 8), 9_467);
        assert_eq!(board.perft_multithread(4, 8), 422_333);
        assert_eq!(board.perft_multithread(5, 8), 15_833_292);
        assert_eq!(board.perft_multithread(6, 8), 706_045_033);
    }

    #[test]
    fn position_5() {
        let mut board =
            Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .expect("test pos 5 is valid");
        assert_eq!(board.perft_multithread(1, 8), 44);
        assert_eq!(board.perft_multithread(2, 8), 1_486);
        assert_eq!(board.perft_multithread(3, 8), 62_379);
        assert_eq!(board.perft_multithread(4, 8), 2_103_487);
        assert_eq!(board.perft_multithread(5, 8), 89_941_194);
    }
    
    #[test]
    fn talkchess() {
        // see https://www.chessprogramming.net/perfect-perft/
        const PERFTS: &[(&str, u8, usize)] = &[
            ("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1_134_888),
            ("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1_015_133),
            ("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1_440_467),
            ("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661_072),
            ("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803_711),
            ("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1_274_206),
            ("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4, 1_720_476),
            ("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6, 3_821_001),
            ("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5, 1_004_658),
            ("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6, 217_342),
            ("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6, 92_683),
            ("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6, 2_217),
            ("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4, 23_527),
            ("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7, 567_584)
        ];

        for (i, (fen, depth, move_count)) in PERFTS.iter().enumerate() {
            let mut board = Board::from_fen(fen).expect("fen is valid");
            assert_eq!(board.perft_multithread(*depth, 8), *move_count, "failed pos {i} with fen {fen}")
        }
    }
}
