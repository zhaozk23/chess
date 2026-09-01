use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::sync::LazyLock;
use std::time::{Duration, Instant};
pub type State = u8;
pub type Pos = u8;
pub type PType = i8;
pub type Pid = u8;
pub type Team = i8;
pub type Depth = u8;
pub type VL = i16;
pub type Hash = i64;
pub type PregenData = u8;
pub type HashFlag = i8;
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct Move {
    pub beg: u8,
    pub end: u8,
}

pub type SearchRet = (Move, VL);
impl Move {
    pub fn new(beg: u8, end: u8) -> Self {
        Self { beg, end }
    }
    pub fn from_ucimove(s: &str) -> Self {
        if s.len() != 4 {
            return Self::new(0, 0);
        }
        let f0 = s.chars().nth(0).unwrap() as u8;
        let r0 = s.chars().nth(1).unwrap() as u8;
        let f1 = s.chars().nth(2).unwrap() as u8;
        let r1 = s.chars().nth(3).unwrap() as u8;
        if f0 < b'a'
            || f0 > b'i'
            || r0 < b'0'
            || r0 > b'9'
            || f1 < b'a'
            || f1 > b'i'
            || r1 < b'0'
            || r1 > b'9'
        {
            return Self::new(0, 0);
        }
        let beg = (9 - (r0 - b'0')) * 9 + (f0 - b'a');
        let end = (9 - (r1 - b'0')) * 9 + (f1 - b'a');
        if beg == end {
            return Self::new(0, 0);
        }
        Self::new(beg, end)
    }
    pub fn to_ucimove(&self) -> String {
        let f0 = (self.beg % 9 + b'a') as char;
        let r0 = (9 - self.beg / 9 + b'0') as char;
        let f1 = (self.end % 9 + b'a') as char;
        let r1 = (9 - self.end / 9 + b'0') as char;
        format!("{}{}{}{}", f0, r0, f1, r1)
    }
}
impl Into<bool> for Move {
    fn into(self) -> bool {
        self.beg != self.end
    }
}
impl Into<u32> for Move {
    fn into(self) -> u32 {
        (self.beg * 100 + self.end) as u32
    }
}
pub type TrickRet = (bool, VL);
pub type Matrix = [PType; 90];
pub type PregenTable = [[PregenData; 1024]; 10];
pub type GenType = u8;

pub const INVALID_POS: Pos = 100;
pub const EXACT: HashFlag = 0;
pub const ALPHA: HashFlag = 1;
pub const BETA: HashFlag = 2;
pub const R_KING: PType = 1;
pub const R_ADVISOR: PType = 2;
pub const R_BISHOP: PType = 3;
pub const R_KNIGHT: PType = 4;
pub const R_ROOK: PType = 5;
pub const R_CANNON: PType = 6;
pub const R_PAWN: PType = 7;
pub const B_KING: PType = -1;
pub const B_ADVISOR: PType = -2;
pub const B_BISHOP: PType = -3;
pub const B_KNIGHT: PType = -4;
pub const B_ROOK: PType = -5;
pub const B_CANNON: PType = -6;
pub const B_PAWN: PType = -7;
pub const R: Team = 1;
pub const B: Team = -1;
pub const INF: VL = 30000;
pub const BAN: VL = 20000;
pub const INVALID_VL: VL = -31000;
pub const SIDE_KEY: Hash = 7655453740479314502;
pub const Q_MAX_DISTANCE: Depth = 8;
pub const FP_MARGIN: VL = 120;
pub const Q_CHECKING_DEPTH: Depth = 4;
pub const Q_DELTA_MARGIN: VL = 80;
pub const NULL_MOVE_R: Depth = 2;
pub const NULL_MOVE_MIN_DEPTH: Depth = 3;
pub const LMR_MIN_DEPTH: Depth = 3;
pub const LMR_MIN_MOVES: i32 = 3;
pub const LMR_BASE: Depth = 1;
pub const QUIET: GenType = 0;
pub const CAPTURE: GenType = 1;
pub const ALL: GenType = 2;
pub const NODE_PV: bool = false;
pub const NODE_CUT: bool = true;
pub const WEIGHTS: [VL; 8] = [0, 30, 2, 2, 4, 10, 5, 1];

fn set_left_4bit(d: &mut PregenData, n: u32) {
    *d |= (n << 4) as u8
}
fn set_right_4bit(d: &mut PregenData, n: u32) {
    *d |= n as u8
}
fn get_left_4bit(d: PregenData) -> u8 {
    d >> 4
}
fn get_right_4bit(d: PregenData) -> u8 {
    d & 0xF
}
fn get_bit(d: u32, i: u32) -> u32 {
    (d >> i) & 1
}
struct Timer {
    beg: Instant,
    limit: Duration,
}
impl Timer {
    pub fn new() -> Self {
        Self {
            beg: Instant::now(),
            limit: Duration::default(),
        }
    }
    pub fn from_millis(limit: u32) -> Self {
        Self {
            beg: Instant::now(),
            limit: Duration::from_millis(limit as u64),
        }
    }
    pub fn time_up(&self) -> bool {
        Instant::now() - self.beg >= self.limit
    }
    pub fn time_up_2x(&self) -> bool {
        Instant::now() - self.beg >= self.limit / 2
    }
    pub fn duration(&self) -> u128 {
        let d = Instant::now() - self.beg;
        d.as_millis()
    }
}
#[derive(Default)]
struct TTEntry {
    key: Hash,
    flag: HashFlag,
    vl: VL,
    depth: Depth,
    move_: Move,
}
/// hash keys for zobrist hashing
/// accessing: HASH_KEYS[size_t(piece + 7)][pos]
pub static HASH_KEYS: LazyLock<[[i64; 90]; 15]> = LazyLock::new(|| {
    let mut rng = StdRng::seed_from_u64(2820795095);
    let mut ret = [[0i64; 90]; 15];
    for row in &mut ret {
        for val in row {
            *val = rng.random(); // 生成 i64 全范围 [MIN, MAX]
        }
    }
    ret[7] = [0; 90]; // 第 7 行（从 0 开始）置零
    ret
});
/// pregen points for rook and cannon non-capture moves
pub static ROOK_PREGEN: LazyLock<PregenTable> = LazyLock::new(|| {
    let mut ret = [[0u8; 1024]; 10];
    for (pos, row) in ret.iter_mut().enumerate() {
        for (bitline, entry) in row.iter_mut().enumerate() {
            for i in pos + 1..=9 {
                if get_bit(bitline as u32, i as u32) == 1 {
                    set_right_4bit(entry, i as u32);
                    break;
                }
            }
            for i in (0..pos).rev() {
                if get_bit(bitline as u32, i as u32) == 1 {
                    set_left_4bit(entry, i as u32);
                    break;
                }
            }
        }
    }
    ret
});
/// pregen points for cannon capture moves
pub static CANNON_PREGEN: LazyLock<PregenTable> = LazyLock::new(|| {
    let mut ret = [[0u8; 1024]; 10];
    for (pos, row) in ret.iter_mut().enumerate() {
        for (bitline, entry) in row.iter_mut().enumerate() {
            let mut left = INVALID_POS;
            let mut right = INVALID_POS;
            let mut i = pos as u8 + 1;
            let mut t = 0;
            loop {
                if i > 9 {
                    set_right_4bit(entry, 0b1111);
                    break;
                }
                if get_bit(bitline as u32, i as u32) > 0 {
                    if t == 0 {
                        t += 1;
                    } else if i < 10 {
                        set_right_4bit(entry, i as u32);
                        break;
                    }
                }
                i += 1;
            }
            i = pos as u8 - 1;
            t = 0;
            loop {
                if i == 0xFF {
                    set_left_4bit(entry, 0b1111);
                    break;
                }
                if get_bit(bitline as u32, i as u32) > 0 {
                    if t == 0 {
                        t += 1;
                    } else if i > 0 {
                        set_left_4bit(entry, i as u32);
                        break;
                    }
                }
                i -= 1;
            }
        }
    }
    ret
});
/// convert fen string to matrix representation
fn fen_to_matrix(fen: &str) -> Matrix {
    let mut matrix = [0i8; 90];
    let mut i = 0;
    let t = "KABNRCPkabnrcp";
    for c in fen.chars() {
        if c == ' ' {
            break;
        }
        if c == '/' {
            continue;
        }
        if c >= '1' && c <= '9' {
            let n = c as u8 - b'0';
            i += n;
            continue;
        }
        if let Some(idx) = t.find(c)
            && i < 90
        {
            matrix[i as usize] = if idx < 7 {
                (idx + 1) as i8
            } else {
                (6 - idx) as i8
            };
            i += 1;
        }
    }
    matrix
}
fn matrix_to_fen(matrix: &Matrix) -> String {
    let mut fen = String::new();
    for i in 0..10 {
        if i > 0 {
            fen.push('/');
        }
        let mut e = 0;
        for j in 0..9 {
            let idx = i * 9 + j;
            let p = matrix[idx];
            if p == 0 {
                e += 1;
                continue;
            }
            if e > 0 {
                fen.push_str(&e.to_string());
                e = 0;
            }
            let c = if p > 0 {
                "KABNRCP".chars().nth((p - 1) as usize).unwrap()
            } else {
                "kabnrcp".chars().nth((-p - 1) as usize).unwrap()
            };
            fen.push(c);
        }
    }
    fen
}
fn get_banner_points(bl: u16, p: Pos, is9: bool, is_rook: bool) -> (Pos, Pos) {
    assert!(p < 90);
    let i = if is9 { p % 9 } else { p / 9 };
    let v = if is_rook {
        ROOK_PREGEN[i as usize][bl as usize]
    } else {
        CANNON_PREGEN[i as usize][bl as usize]
    };
    let v1 = get_left_4bit(v);
    let v2 = get_right_4bit(v);
    let l = if v1 == 0b1111 { INVALID_POS } else { v1 };
    if is9 {
        let r = if v2 == 0b1111 {
            INVALID_POS
        } else if v2 != 9 {
            v2
        } else {
            8
        };
        (p / 9 * 9 + l, p / 9 * 9 + r)
    } else {
        let r = if v2 == 0b1111 { INVALID_POS } else { v2 };
        (p % 9 + l * 9, p % 9 + r * 9)
    }
}
pub fn rook9(bl: u16, p: Pos) -> (Pos, Pos) {
    get_banner_points(bl, p, true, true)
}
pub fn rook10(bl: u16, p: Pos) -> (Pos, Pos) {
    get_banner_points(bl, p, false, true)
}
pub fn cannon9(bl: u16, p: Pos) -> (Pos, Pos) {
    get_banner_points(bl, p, true, false)
}
pub fn cannon10(bl: u16, p: Pos) -> (Pos, Pos) {
    get_banner_points(bl, p, false, false)
}
const PST_ADVISOR: [VL; 90] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 120, 0, 120, 0, 0, 0, 0, 0, 0, 0, 125, 0, 0, 0, 0, 0, 0, 0, 120, 0, 120, 0, 0, 0,
];
const PST_BISHOP: [VL; 90] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 110, 0, 0, 0, 110, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 125, 0, 0, 0, 130, 0, 0, 0, 125, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 120, 0, 0, 0, 120, 0, 0,
];
const PST_PAWN: [VL; 90] = [
    70, 70, 80, 120, 150, 120, 80, 70, 70, 80, 80, 90, 130, 160, 130, 90, 80, 80, 70, 75, 85, 120,
    150, 120, 85, 75, 70, 60, 65, 75, 90, 100, 90, 75, 65, 60, 55, 60, 70, 80, 90, 80, 70, 60, 55,
    35, 35, 40, 45, 50, 45, 40, 35, 35, 30, 30, 35, 40, 45, 40, 35, 30, 30, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const PST_KNIGHT: [VL; 90] = [
    250, 255, 260, 265, 260, 265, 260, 255, 250, 255, 265, 275, 280, 275, 280, 275, 265, 255, 260,
    275, 285, 290, 285, 290, 285, 275, 260, 265, 280, 295, 300, 295, 300, 295, 280, 265, 265, 285,
    300, 305, 300, 305, 300, 285, 265, 260, 285, 295, 305, 300, 305, 295, 285, 260, 255, 280, 290,
    300, 295, 300, 290, 280, 255, 250, 270, 285, 290, 285, 290, 285, 270, 250, 245, 265, 275, 285,
    275, 285, 275, 265, 245, 240, 260, 270, 280, 270, 280, 270, 260, 240,
];
const PST_ROOK: [VL; 90] = [
    610, 620, 620, 630, 640, 630, 620, 620, 610, 615, 625, 625, 635, 645, 635, 625, 625, 615, 610,
    620, 620, 630, 640, 630, 620, 620, 610, 615, 625, 625, 635, 645, 635, 625, 625, 615, 605, 615,
    615, 625, 635, 625, 615, 615, 605, 600, 610, 610, 620, 630, 620, 610, 610, 600, 600, 610, 610,
    620, 630, 620, 610, 610, 600, 595, 605, 605, 615, 625, 615, 605, 605, 595, 595, 605, 605, 615,
    620, 615, 605, 605, 595, 590, 600, 600, 610, 615, 610, 600, 600, 590,
];
const PST_CANNON: [VL; 90] = [
    300, 300, 305, 310, 315, 310, 305, 300, 300, 300, 300, 305, 310, 315, 310, 305, 300, 300, 295,
    295, 300, 305, 315, 305, 300, 295, 295, 295, 295, 300, 305, 310, 305, 300, 295, 295, 290, 290,
    295, 300, 310, 300, 295, 290, 290, 290, 290, 295, 300, 310, 300, 295, 290, 290, 290, 295, 295,
    300, 310, 300, 295, 295, 290, 290, 295, 295, 300, 315, 300, 295, 295, 290, 285, 290, 290, 295,
    300, 295, 290, 290, 285, 285, 290, 290, 295, 295, 295, 290, 290, 285,
];
pub fn pst(team: Team, abs_type: PType, pos: Pos) -> VL {
    let sq = if team == R { pos } else { 89 - pos };
    match abs_type {
        R_ADVISOR => PST_ADVISOR[sq as usize],
        R_BISHOP => PST_BISHOP[sq as usize],
        R_PAWN => PST_PAWN[sq as usize],
        R_KNIGHT => PST_KNIGHT[sq as usize],
        R_ROOK => PST_ROOK[sq as usize],
        R_CANNON => PST_CANNON[sq as usize],
        _ => 0,
    }
}
