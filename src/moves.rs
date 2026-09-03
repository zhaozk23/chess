use crate::common::*;
use crate::heuristic::*;
use crate::position::*;
impl Position {
    fn gen_king(&self, g: GenType, pos: Pos) -> Vec<Move> {
        let mut ret = Vec::new();
        ret.reserve(4);
        if (2 < pos && pos < 15 || 65 < pos && pos < 78) && self.target_chk(pos + 9, g) {
            ret.push(Move::new(pos, pos + 9));
        }
        if (11 < pos && pos < 24 || 74 < pos && pos < 87) && self.target_chk(pos - 9, g) {
            ret.push(Move::new(pos, pos - 9));
        }
        if (pos % 9 == 4 || pos % 9 == 5) && self.target_chk(pos - 1, g) {
            ret.push(Move::new(pos, pos - 1));
        }
        if (pos % 9 == 4 || pos % 9 == 3) && self.target_chk(pos + 1, g) {
            ret.push(Move::new(pos, pos + 1));
        }
        ret
    }
    fn gen_advisor(&self, g: GenType, pos: Pos) -> Vec<Move> {
        if pos == 13 || pos == 76 {
            let mut ret = Vec::new();
            ret.reserve(4);
            if self.target_chk(pos - 10, g) {
                ret.push(Move::new(pos, pos - 10));
            }
            if self.target_chk(pos + 10, g) {
                ret.push(Move::new(pos, pos + 10));
            }
            if self.target_chk(pos - 8, g) {
                ret.push(Move::new(pos, pos - 8));
            }
            if self.target_chk(pos + 8, g) {
                ret.push(Move::new(pos, pos + 8));
            }
            return ret;
        } else if pos < 24 && self.target_chk(13, g) {
            return vec![Move::new(pos, 13)];
        } else if pos > 65 && self.target_chk(76, g) {
            return vec![Move::new(pos, 76)];
        } else {
            return Vec::new();
        }
    }
    fn gen_bishop(&self, g: GenType, pos: Pos) -> Vec<Move> {
        let mut ret = Vec::new();
        ret.reserve(4);
        if pos / 9 == 0 || pos / 9 == 5 || pos / 9 == 7 || pos / 9 == 3 {
            if self.get_piece(pos + 10) == 0 && self.target_chk(pos + 16, g) {
                ret.push(Move::new(pos, pos + 16));
            }
            if self.get_piece(pos + 12) == 0 && self.target_chk(pos + 20, g) {
                ret.push(Move::new(pos, pos + 20));
            }
        }
        if pos / 9 == 4 || pos / 9 == 9 || pos / 9 == 7 || pos / 9 == 3 {
            if self.get_piece(pos - 10) == 0 && self.target_chk(pos - 16, g) {
                ret.push(Move::new(pos, pos - 16));
            }
            if self.get_piece(pos - 12) == 0 && self.target_chk(pos - 20, g) {
                ret.push(Move::new(pos, pos - 20));
            }
        }
        ret
    }
    fn gen_knight(&self, g: GenType, pos: Pos) -> Vec<Move> {
        let mut ret = Vec::new();
        ret.reserve(8);
        if pos > 17 && self.get_piece(pos - 9) == 0 {
            if pos % 9 != 0 && self.target_chk(pos - 19, g) {
                ret.push(Move::new(pos, pos - 19));
            }
            if pos % 9 != 0 && self.target_chk(pos - 17, g) {
                ret.push(Move::new(pos, pos - 17));
            }
        }
        if pos < 72 && self.get_piece(pos + 9) == 0 {
            if pos % 9 != 0 && self.target_chk(pos + 19, g) {
                ret.push(Move::new(pos, pos + 19));
            }
            if pos % 9 != 0 && self.target_chk(pos + 17, g) {
                ret.push(Move::new(pos, pos + 17));
            }
        }
        if pos % 9 > 1 && self.get_piece(pos - 1) == 0 {
            if pos / 9 != 0 && self.target_chk(pos - 11, g) {
                ret.push(Move::new(pos, pos - 11));
            }
            if pos / 9 != 9 && self.target_chk(pos + 7, g) {
                ret.push(Move::new(pos, pos + 7));
            }
        }
        if pos % 9 < 7 && self.get_piece(pos + 1) == 0 {
            if pos / 9 != 0 && self.target_chk(pos + 11, g) {
                ret.push(Move::new(pos, pos + 11));
            }
            if pos / 9 != 9 && self.target_chk(pos - 7, g) {
                ret.push(Move::new(pos, pos - 7));
            }
        }
        ret
    }
    fn gen_rook(&self, g: GenType, pos: Pos) -> Vec<Move> {
        let mut ret = Vec::new();
        ret.reserve(24);
        let (l, r) = rook9(self.get_bl9(pos), pos);
        let (u, b) = rook10(self.get_bl10(pos), pos);
        if g != CAPTURE {
            for p in (l + 1..pos).rev() {
                ret.push(Move::new(pos, p));
            }
            if self.get_piece(l) == 0 {
                ret.push(Move::new(pos, l));
            }
            for p in pos + 1..r {
                ret.push(Move::new(pos, p));
            }
            if self.get_piece(r) == 0 {
                ret.push(Move::new(pos, r));
            }
            for p in (u + 1..pos - 8).rev().step_by(9) {
                ret.push(Move::new(pos, p));
            }
            if self.get_piece(u) == 0 {
                ret.push(Move::new(pos, u));
            }
            for p in (pos + 9..b).step_by(9) {
                ret.push(Move::new(pos, p));
            }
            if self.get_piece(b) == 0 {
                ret.push(Move::new(pos, b));
            }
        }
        if g != QUIET {
            if self.target_chk(l, CAPTURE) {
                ret.push(Move::new(pos, l));
            }
            if self.target_chk(r, CAPTURE) {
                ret.push(Move::new(pos, r));
            }
            if self.target_chk(u, CAPTURE) {
                ret.push(Move::new(pos, u));
            }
            if self.target_chk(b, CAPTURE) {
                ret.push(Move::new(pos, b));
            }
        }
        ret
    }
    fn gen_cannon(&self, g: GenType, pos: Pos) -> Vec<Move> {
        if g == QUIET {
            return self.gen_rook(g, pos);
        }
        let mut ret = Vec::new();
        ret.reserve(4);
        let (l, r) = cannon9(self.get_bl9(pos), pos);
        let (u, b) = cannon10(self.get_bl10(pos), pos);
        if l < INVALID_POS && self.target_chk(l, CAPTURE) {
            ret.push(Move::new(pos, l));
        }
        if r < INVALID_POS && self.target_chk(r, CAPTURE) {
            ret.push(Move::new(pos, r));
        }
        if u < INVALID_POS && self.target_chk(u, CAPTURE) {
            ret.push(Move::new(pos, u));
        }
        if b < INVALID_POS && self.target_chk(b, CAPTURE) {
            ret.push(Move::new(pos, b));
        }
        if g == ALL {
            let qs = self.gen_rook(QUIET, pos);
            ret.extend(qs);
        }
        ret
    }
    fn gen_pawn(&self, g: GenType, pos: Pos) -> Vec<Move> {
        let mut ret = Vec::new();
        ret.reserve(3);
        let target = pos as i8 - 9 * self.team;
        if 0 <= target && target < 90 && self.target_chk(target as Pos, g) {
            ret.push(Move::new(pos, target as u8));
        }
        if (pos / 9 < 5 && self.team == R) || (pos / 9 > 4 && self.team == B) {
            if pos % 9 != 0 && self.target_chk(pos - 1, g) {
                ret.push(Move::new(pos, pos - 1));
            }
            if pos % 9 != 8 && self.target_chk(pos + 1, g) {
                ret.push(Move::new(pos, pos + 1));
            }
        }
        ret
    }
    fn gen_moves(&self, g: GenType) -> Vec<Move> {
        let mut ret = Vec::new();
        ret.reserve(16);
        for p in self.get_pos_list() {
            let t = self.get_piece(p).abs();
            match t {
                R_KING => ret.extend(self.gen_king(g, p)),
                R_ADVISOR => ret.extend(self.gen_advisor(g, p)),
                R_BISHOP => ret.extend(self.gen_bishop(g, p)),
                R_KNIGHT => ret.extend(self.gen_knight(g, p)),
                R_ROOK => ret.extend(self.gen_rook(g, p)),
                R_CANNON => ret.extend(self.gen_cannon(g, p)),
                _ => unreachable!(),
            }
        }
        ret
    }
    pub fn gen_capture_moves(&self) -> Vec<Move> {
        self.gen_moves(CAPTURE)
    }
    pub fn gen_quiet_moves(&self) -> Vec<Move> {
        self.gen_moves(QUIET)
    }
    pub fn gen_all_moves(&self) -> Vec<Move> {
        self.gen_moves(ALL)
    }
}

#[derive(Default)]
pub struct MovePicker {
    starts: [Move; 3],
    moves: Vec<Move>,
    i: isize,
}
impl MovePicker {
    // Maybe put it a different place
    fn order_rest(moves: Vec<Move>, p: &Position, h: &HistoryTable) -> Vec<Move> {
        let (mut left, mut right): (Vec<_>, Vec<_>) =
            moves.into_iter().partition(|m| p.get_piece(m.end) != 0);
        p.mvvlva_sort(&mut left);
        h.sort_by_history(&mut right, p.team);
        left.extend(right);
        left
    }
    pub fn new(depth: Depth, k: &KillerTable, tt: TT, p: &Position) -> Self {
        let mut starts = [Move::default(); 3];
        starts[0] = tt.get_move(p.key);
        let killers = k.get(depth);
        let c1 = killers[0].into() && killers[0] != starts[0];
        let c2 = killers[1].into() && killers[1] != starts[1];
        if c1 && p.legal_move(killers[0]) {
            starts[1] = killers[0]
        }
        if c2 && p.legal_move(killers[1]) {
            starts[2] = killers[1]
        }
        Self {
            starts,
            ..Default::default()
        }
    }
    pub fn next(&mut self, p: &Position, h: &HistoryTable) -> Move {
        if self.i >= 0 && self.i < 3 {
            let m = self.starts[self.i as usize];
            self.i += 1;
            return if m.into() { m } else { self.next(p, h) };
        }
        if self.i == 3 {
            let moves = p.gen_all_moves();
            self.moves = MovePicker::order_rest(moves, p, h);
            self.i = -1;
            return self.next(p, h);
        }
        let idx = -self.i - 1;
        assert!(idx >= 0);
        let idx = idx as usize;
        if idx < self.moves.len() {
            let m = self.moves[idx];
            let c = self.starts.contains(&m);
            self.i -= 1;
            return if c { self.next(p, h) } else { m };
        }
        Move::default()
    }
}
