use crate::common::*;
use crate::position::*;

const TEMPO: VL = 8;
// Maybe usize?
const HANG_PEN: [i32; 8] = [0, 0, 25, 25, 50, 110, 55, 8];
const MOB_ROOK: i32 = 2;
const MOB_CANNON: i32 = 2;
const MOB_KNIGHT: i32 = 4;

struct SideInfo {
    advisors: usize,
    bishops: usize,
    knights: usize,
    rooks: usize,
    cannons: usize,
    pawns: usize,
    over_river: usize,
    mobility: i32,
    open_file: i32,
    threat: i32,
    pawn_files: u32,
    king: Pos,
    cannon_sq: [Pos; 2],
}

impl Default for SideInfo {
    fn default() -> Self {
        Self {
            advisors: 0,
            bishops: 0,
            knights: 0,
            rooks: 0,
            cannons: 0,
            pawns: 0,
            over_river: 0,
            mobility: 0,
            open_file: 0,
            threat: 0,
            pawn_files: 0,
            king: INVALID_POS,
            cannon_sq: [INVALID_POS; 2],
        }
    }
}
fn crossed_river(team: Team, pos: Pos) -> bool {
    if team == R { pos / 9 < 5 } else { pos / 9 > 4 }
}
fn attacks_sq(p: &Position, by: Team, sq: Pos) -> bool {
    let f = sq % 9;
    if p.get_piece((sq as i8 + 9 * by) as Pos) * by == R_PAWN {
        return true;
    }
    if crossed_river(by, sq) {
        if f != 0 && p.get_piece(sq - 1) * by == R_PAWN {
            return true;
        }
        if f != 0 && p.get_piece(sq + 1) * by == R_PAWN {
            return true;
        }
    }
    if f != 0 {
        if p.get_piece(sq - 10) * by == R_ADVISOR {
            return true;
        }
        if p.get_piece(sq + 8) * by == R_ADVISOR {
            return true;
        }
    }
    if f != 8 {
        if p.get_piece(sq - 8) * by == R_ADVISOR {
            return true;
        }
        if p.get_piece(sq + 10) * by == R_ADVISOR {
            return true;
        }
    }
    if f >= 2 {
        if p.get_piece(sq - 10) == 0 && p.get_piece(sq - 20) * by == R_BISHOP {
            return true;
        }
        if p.get_piece(sq + 8) == 0 && p.get_piece(sq + 16) * by == R_BISHOP {
            return true;
        }
    }
    if f <= 6 {
        if p.get_piece(sq - 8) == 0 && p.get_piece(sq - 16) * by == R_BISHOP {
            return true;
        }
        if p.get_piece(sq + 10) == 0 && p.get_piece(sq + 20) * by == R_BISHOP {
            return true;
        }
    }
    if p.get_piece(sq - 10) == 0 {
        if f >= 1 && p.get_piece(sq - 19) * by == R_KNIGHT {
            return true;
        }
        if f <= 6 && p.get_piece(sq - 11) * by == R_KNIGHT {
            return true;
        }
    }
    if p.get_piece(sq + 10) == 0 {
        if f <= 7 && p.get_piece(sq + 19) * by == R_KNIGHT {
            return true;
        }
        if f >= 2 && p.get_piece(sq + 11) * by == R_KNIGHT {
            return true;
        }
    }
    if p.get_piece(sq + 8) == 0 {
        if f >= 1 && p.get_piece(sq + 17) * by == R_KNIGHT {
            return true;
        }
        if f <= 6 && p.get_piece(sq + 7) * by == R_KNIGHT {
            return true;
        }
    }
    if p.get_piece(sq - 8) == 0 {
        if f <= 7 && p.get_piece(sq - 17) * by == R_KNIGHT {
            return true;
        }
        if f >= 2 && p.get_piece(sq - 7) * by == R_KNIGHT {
            return true;
        }
    }
    let (rl, rr) = rook9(p.get_bl9(sq), sq);
    let (rt, rb) = rook10(p.get_bl10(sq), sq);
    if p.get_piece(rl) * by == R_ROOK
        || p.get_piece(rr) * by == R_ROOK
        || p.get_piece(rt) * by == R_ROOK
        || p.get_piece(rb) * by == R_ROOK
    {
        return true;
    }
    let (cl, cr) = cannon9(p.get_bl9(sq), sq);
    let (ct, cb) = cannon10(p.get_bl10(sq), sq);
    if p.get_piece(rl) * by == R_CANNON
        || p.get_piece(rr) * by == R_CANNON
        || p.get_piece(rt) * by == R_CANNON
        || p.get_piece(rb) * by == R_CANNON
    {
        return true;
    }

    if f != 0 && p.get_piece(sq - 1) * by == R_KING {
        return true;
    }
    if f != 8 && p.get_piece(sq + 1) * by == R_KING {
        return true;
    }
    if p.get_piece(sq - 9) * by == R_KING {
        return true;
    }
    if p.get_piece(sq + 9) * by == R_KING {
        return true;
    }
    false
}

fn rook_mobility(p: &Position, team: Team, pos: Pos) -> i32 {
    let (rl, rr) = rook9(p.get_bl9(pos), pos);
    let (rt, rb) = rook10(p.get_bl10(pos), pos);
    let mut mob = rr as i32 - rl as i32 - 2 + (rb as i32 - rt as i32) / 9 - 2;
    if p.get_piece(rl) * team <= 0 {
        mob += 1;
    }
    if p.get_piece(rr) * team <= 0 {
        mob += 1;
    }
    if p.get_piece(rt) * team <= 0 {
        mob += 1;
    }
    if p.get_piece(rb) * team <= 0 {
        mob += 1;
    }
    mob
}

fn cannon_mobility(p: &Position, team: Team, pos: Pos) -> i32 {
    let (rl, rr) = rook9(p.get_bl9(pos), pos);
    let (rt, rb) = rook10(p.get_bl10(pos), pos);
    let mut mob = rr as i32 - rl as i32 - 2 + (rb as i32 - rt as i32) / 9 - 2;
    if p.get_piece(rl) * team <= 0 {
        mob += 1;
    }
    if p.get_piece(rr) * team <= 0 {
        mob += 1;
    }
    if p.get_piece(rt) * team <= 0 {
        mob += 1;
    }
    if p.get_piece(rb) * team <= 0 {
        mob += 1;
    }
    let (cl, cr) = cannon9(p.get_bl9(pos), pos);
    let (ct, cb) = cannon10(p.get_bl10(pos), pos);
    if cl < 90 && p.get_piece(cl) * team < 0 {
        mob += 1;
    }
    if cr < 90 && p.get_piece(cr) * team < 0 {
        mob += 1;
    }
    if ct < 90 && p.get_piece(ct) * team < 0 {
        mob += 1;
    }
    if cb < 90 && p.get_piece(cb) * team < 0 {
        mob += 1;
    }
    mob
}

fn knight_mobility(p: &Position, team: Team, pos: Pos) -> i32 {
    let f = pos / 9;
    let mut mob = 0;
    let ok = |dst: i32| {
        return dst >= 0 && dst < 90 && p.get_piece(dst as Pos) * team <= 0;
    };
    if pos >= 9 && p.get_piece(pos - 9) == 0 {
        if f >= 1 && ok(pos as i32 - 19) {
            mob += 1;
        }
        if f <= 7 && ok(pos as i32 - 17) {
            mob += 1;
        }
    }
    if pos <= 80 && p.get_piece(pos + 9) == 0 {
        if f >= 1 && ok(pos as i32 + 17) {
            mob += 1;
        }
        if f <= 7 && ok(pos as i32 + 19) {
            mob += 1;
        }
    }
    if f >= 2 && p.get_piece(pos - 1) == 0 {
        if ok(pos as i32 - 11) {
            mob += 1;
        }
        if ok(pos as i32 + 7) {
            mob += 1;
        }
    }
    if f <= 6 && p.get_piece(pos + 1) == 0 {
        if ok(pos as i32 + 11) {
            mob += 1;
        }
        if ok(pos as i32 - 7) {
            mob += 1;
        }
    }
    mob
}

fn hollow_cannon(p: &Position, ca: Pos, king: Pos) -> bool {
    if ca % 9 != king % 9 {
        return false;
    }
    let lo = ca.min(king);
    let hi = ca.max(king);
    for pos in (lo + 9..hi).step_by(9) {
        if p.board[pos as usize] != 0 {
            return false;
        }
    }
    true
}

impl SideInfo {
    fn collect_side(p: &Position, team: Team) -> Self {
        let mut s = SideInfo::default();
        let list = if team == R {
            &p.pos_list_r
        } else {
            &p.pos_list_b
        };
        for &pos in list {
            let over = crossed_river(team, pos);
            let id = p.board[pos as usize].abs();
            match id {
                R_KING => s.king = pos,
                R_ADVISOR => s.advisors += 1,
                R_BISHOP => s.bishops += 1,
                R_KNIGHT => {
                    s.knights += 1;
                    if over {
                        s.over_river += 2;
                    }
                }
                R_ROOK => {
                    s.rooks += 1;
                    if over {
                        s.over_river += 3;
                    }
                }
                R_CANNON => {
                    s.cannon_sq[s.cannons.min(1)] = pos;
                    s.cannons += 1;
                    if over {
                        s.over_river += 2;
                    }
                }
                R_PAWN => {
                    s.pawns += 1;
                    s.pawn_files |= 1 << (pos % 9);
                    if over {
                        s.over_river += 1;
                    }
                }
                _ => {}
            }
        }
        s
    }
    fn collect_positional(&mut self, p: &Position, team: Team, opp: &SideInfo) {
        let list = if team == R {
            &p.pos_list_r
        } else {
            &p.pos_list_b
        };
        for &pos in list {
            let id = p.board[pos as usize].abs();
            match id {
                R_KNIGHT => self.mobility += MOB_KNIGHT * knight_mobility(p, team, pos),
                R_ROOK => {
                    self.mobility += MOB_ROOK * rook_mobility(p, team, pos);
                    if self.pawn_files & (1 << (pos % 9)) == 0 {
                        self.open_file += 10;
                        if opp.pawn_files & (1 << (pos % 9)) == 0 {
                            self.open_file += 8;
                        }
                    }
                }
                R_CANNON => self.mobility += MOB_CANNON * cannon_mobility(p, team, pos),
                _ => {}
            }
            if id != R_KING && attacks_sq(p, -team, pos) && !attacks_sq(p, team, pos) {
                self.threat += HANG_PEN[id as usize];
            }
        }
    }
    fn shape_bonus(&self, opp: &SideInfo, p: &Position, team: Team) -> i32 {
        let mut vl = 0;
        if self.advisors == 2 && self.bishops == 2 {
            vl += 15;
        }
        if opp.advisors == 0 && self.rooks >= 2 {
            vl += 30;
        } else if opp.advisors <= 1 && self.rooks >= 1 {
            vl += 10;
        }
        if opp.bishops == 0 && self.cannons >= 1 {
            vl += 20;
        } else if opp.bishops <= 1 && self.cannons >= 1 {
            vl += 8;
        }
        let heart = if team == R { 76 } else { 13 };
        if p.board[heart] * team == R_KNIGHT {
            vl -= 50;
        }
        for c in self.cannon_sq {
            if c < 90 && opp.king < 90 && hollow_cannon(p, c, opp.king) {
                let m = (self.rooks + self.knights + self.cannons) as i32;
                vl += 20 + m * 8;
                break;
            }
        }
        if self.over_river > opp.over_river {
            vl += i32::min(24, 5 * (self.over_river as i32 - opp.over_river as i32));
        }
        vl
    }
}
pub fn evaluate(p: &Position) -> VL {
    let mut vl = p.eval as i32;
    let mut red = SideInfo::collect_side(p, R);
    let mut black = SideInfo::collect_side(p, B);
    red.collect_positional(p, R, &black);
    black.collect_positional(p, B, &red);
    vl += red.mobility - black.mobility;
    vl += red.open_file - black.open_file;
    vl -= red.threat - black.threat;
    vl += red.shape_bonus(&black, p, R) - black.shape_bonus(&red, p, B);
    vl += if p.team == R {
        TEMPO as i32
    } else {
        -TEMPO as i32
    };
    (vl * p.team as i32) as VL
}
