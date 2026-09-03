use crate::common::*;
use crate::evaluate::*;
use crate::heuristic::*;
use crate::moves::*;
use crate::position::*;

struct Search {
    stop: State,
    max_depth: Depth,
    duration: u32,
    distance: Depth,
    uci: bool,
}
impl Default for Search {
    fn default() -> Self {
        Self {
            stop: 0,
            max_depth: 20,
            duration: 1000,
            distance: 0,
            uci: false,
        }
    }
}
impl Search {
    pub fn search(&mut self, p: &Position, tt: &TT) -> (Move, VL) {
        let timer = Timer::from_millis(self.duration);
        let mut vl = -INF;
        let mut depth = 1;
        while !timer.time_up_2x() {
            vl = self.search_vl(depth, -INF, INF, NODE_PV, false, p.in_check());
            let mv = tt.get_move(p.key);
            print!("Info depth {depth} score");
            if self.uci {
                print!("cp ");
            }
            print!("{vl} time {}", timer.duration());
            if mv.into() {
                print!(" pv {}", mv.to_ucimove());
            }
            println!();
            if depth >= self.max_depth {
                break;
            }
            if self.stop > 0 {
                self.stop -= 1;
                break;
            }
            depth += 1;
        }
        let mv = tt.get_move(p.key);
        (mv, vl)
    }
    fn search_vl(
        &mut self,
        depth: Depth,
        mut a: VL,
        b: VL,
        is_cut: bool,
        ban_null: bool,
        checking: bool,
        tt: &mut TT,
        p: &mut Position,
    ) -> VL {
        if depth == 0 {
            return search_q(a, b, Q_MAX_DISTANCE, checking);
        }
        let old_alpha = a;
        let mut vlbest = -INF;
        let mut move_best = Move::default();
        let vl_hash = tt.get_vl(p.key, depth, a, b);
        if vl_hash != INVALID_VL {
            return vl_hash;
        }
        if !checking {
            let vl = evaluate(p);
            if depth <= 2 && depth >= NULL_MOVE_MIN_DEPTH && null_ok(p) {
                let r = NULL_MOVE_R + depth / 6;
                let nd = if depth > r + 1 { depth - r - 1 } else { 0 };
                p.do_null();
                self.distance += 1;
                let vl_null = -self.search_vl(nd, -b, -b + 1, NODE_CUT, true, p.in_check(), tt, p);
                self.distance -= 1;
                p.do_null();
                if vl_null >= b {
                    tt.set(p.key, BETA, depth, Move::default(), vl_null);
                    return vl_null;
                }
            }
        }
        if p.is_repeat() {
            return -INF;
        }
        let mp = MovePicker::new(depth, k, tt, p);
        let mut mv_num = 0;
        let mut mv = mp.next(p, h);
        while mv.into() {
            assert!(p.board[mv.end as usize].abs() != R_KING && p.board[mv.beg as usize] != 0);
            let capture = p.get_piece(mv.end);
            p.do_move(mv);
            self.distance += 1;
            let check = p.in_check();
            mark_checking_move(check);

            // lmr
            let n_depth = depth - 1;
            let mut reduction = 0;
            let c1 = !checking && capture == 0 && !check;
            let c2 = depth >= LMR_MIN_DEPTH && mv_num >= LMR_MIN_MOVES;
            if c1 && c2 {
                reduction = LMR_BASE;
                if depth >= 6 {
                    reduction += 1;
                }
                if mv_num >= 6 {
                    reduction += 1;
                }
                if is_cut {
                    reduction += 1;
                }
                if reduction > n_depth {
                    reduction = n_depth;
                }
            }
            let lmr_depth = n_depth - reduction;

            // pvs
            let mut vl = -INF;
            if !is_cut && vlbest == -INF {
                vl = -self.search_vl(n_depth, -b, -a, NODE_PV, false, check, tt, p);
            } else if is_cut {
                vl = -self.search_vl(lmr_depth, -b, -b + 1, NODE_CUT, ban_null, check, tt, p);
                if reduction > 0 && vl >= b {
                    vl = -self.search_vl(n_depth, -b, -b + 1, NODE_CUT, ban_null, check, tt, p);
                }
            } else {
                vl = -self.search_vl(lmr_depth, -a - 1, a, NODE_CUT, false, check, tt, p);
                if reduction > 0 && vl > a {
                    vl = -self.search_vl(n_depth, -a - 1, -a, NODE_CUT, false, check, tt, p);
                }
                if a < vl && vl < b {
                    vl = -self.search_vl(n_depth, -b, -a, NODE_PV, false, check, tt, p);
                }
            }
            p.undo_move();
            self.distance -= 1;
            if vl > vlbest {
                move_best = mv;
                vlbest = vl;
                a = a.max(vl);
                if vl >= b {
                    break;
                }
            }
            mv = mp.next(p, h);
            mv_num += 1;
        }
        if move_best.into() {
            let mv_type = if vlbest >= b {
                BETA
            } else if vlbest <= old_alpha {
                ALPHA
            } else {
                EXACT
            };
            if mv_type != ALPHA {
                // k.set
            }
            // histoty_table.set
            tt.set(p.key, mv_type, depth, move_best, vlbest);
        }
        return if vlbest != -INF {
            vlbest
        } else {
            vlbest + self.distance as i16
        };
    }
}

fn search_q(a: VL, b: VL, depth: Depth, checking: bool) -> VL {

    todo!()
}
fn null_ok(p: &Position) -> bool {
    p.get_pos_list().len() > 0
}
fn mark_checking_move(checking: bool) {
    if checking
}
