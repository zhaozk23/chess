use crate::common::*;
use crate::position::*;

pub struct HistoryTable {
    table_r: [[u32; 90]; 90],
    table_b: [[u32; 90]; 90],
}
impl HistoryTable {
    pub fn new() -> Self {
        Self {
            table_r: [[0; 90]; 90],
            table_b: [[0; 90]; 90],
        }
    }
    pub fn set(&mut self, mv: Move, team: Team, depth: Depth) {
        let beg = mv.beg as usize;
        let end = mv.end as usize;
        if team == R {
            self.table_r[beg][end] += (depth * depth) as u32;
        } else {
            self.table_b[beg][end] += (depth * depth) as u32;
        }
    }
}
pub fn sort_by_history(h: &HistoryTable, moves: &mut Vec<Move>, team: Team) {
    let t = if team == R { h.table_r } else { h.table_b };
    moves.sort_by(|a, b| {
        let ha = t[a.beg as usize][a.end as usize];
        let hb = t[b.beg as usize][b.end as usize];
        if ha != hb {
            hb.cmp(&ha)
        } else if a.beg != b.beg {
            a.beg.cmp(&b.beg)
        } else {
            a.end.cmp(&b.end)
        }
    });
}
pub struct KillerTable([[Move; 2]; 128]);
impl KillerTable {
    pub fn new() -> Self {
        Self([[Move::default(); 2]; 128])
    }
    pub fn set(&mut self, mv: Move, d: Depth) {
        let d = d as usize;
        if mv == self.0[d][0] {
            return;
        }
        self.0[d][1] = self.0[d][0];
        self.0[d][0] = mv;
    }
    pub fn get(&self, d: Depth) -> [Move; 2] {
        self.0[d as usize]
    }
}
struct TT {
    table: Vec<TTEntry>,
    size: u32,
    mask: u32,
}
impl TT {
    pub fn init(&mut self, size: u32) {
        self.table.clear();
        self.table.resize_with(1 << size, || TTEntry::default());
        self.size = size;
        self.mask = (1 << size) - 1;
    }
    pub fn set(&mut self, key: Hash, flag: HashFlag, depth: Depth, mv: Move, vl: VL) {
        let e = &mut self.table[(key & self.mask as i64) as usize];
        if e.key == 0 {
            e.key = key;
            e.flag = flag;
            e.depth = depth;
            e.mv = mv;
            e.vl = vl;
        } else if e.key == key {
            if depth >= e.depth {
                e.flag = flag;
                e.depth = depth;
                e.mv = mv;
                e.vl = vl;
            }
        } else {
            e.key = key;
            e.flag = flag;
            e.depth = depth;
            e.mv = mv;
            e.vl = vl;
        }
    }
    pub fn get_vl(&self, key: Hash, depth: Depth, alpha: VL, beta: VL) -> VL {
        let e = &self.table[(key & self.mask as i64) as usize];
        if e.key != key || e.depth < depth {
            INVALID_VL
        } else if e.flag == EXACT {
            e.vl
        } else if e.flag == ALPHA && e.vl <= alpha {
            e.vl
        } else if e.flag == BETA && e.vl >= beta {
            e.vl
        } else {
            INVALID_VL
        }
    }
    pub fn get_move(&self, key: Hash) -> Move {
        let e = &self.table[(key & self.mask as i64) as usize];
        if e.key != key { Move::default() } else { e.mv }
    }
}
