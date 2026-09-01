use crate::common::*;
struct History {
    checkings: Vec<bool>,
    moves: Vec<Move>,
    captures: Vec<PType>,
    keys: Vec<Hash>,
}
pub struct Position {
    board: Matrix,
    team: Team,
    key: Hash,
    eval: VL,
    history: History,

    pos_list_r: Vec<Pos>,
    pos_list_b: Vec<Pos>,
    pos_pid_r: [Pid; 90],
    pos_pid_b: [Pid; 90],
    bl10_items: [u16; 10],
    bl9_items: [u16; 9],
}
impl Position {
    pub fn eval_reset(&mut self) {
        self.eval = 0;
    }
    pub fn eval_add(&mut self, team: Team, p: PType, pos: Pos) {
        self.eval += team as i16 * pst(team, p.abs(), pos);
    }
    pub fn eval_remove(&mut self, team: Team, p: PType, pos: Pos) {
        self.eval -= team as i16 * pst(team, p.abs(), pos);
    }
    pub fn eval_slide(&mut self, team: Team, p: PType, pos_from: Pos, pos_to: Pos) {
        self.eval_remove(team, p, pos_from);
        self.eval_add(team, p, pos_to);
    }
    pub fn init(&mut self, board: &Matrix, team: Team) {
        self.history.moves.clear();
        self.history.captures.clear();
        self.history.checkings.clear();
        self.history.keys.clear();
        self.pos_pid_r.fill(0);
        self.pos_pid_b.fill(0);
        self.pos_list_b.clear();
        self.pos_list_r.clear();
        self.bl10_items.fill(0);
        self.bl9_items.fill(0);
        self.key = 0;
        self.board = *board;
        self.team = team;
        if team == B {
            self.key ^= SIDE_KEY;
        }
        // init king pos
        for i in 0..90 {
            if board[i] == R_KING {
                self.pos_pid_r[i] = 0;
                self.pos_list_r.push(i as Pos);
            } else if board[i] == B_KING {
                self.pos_pid_b[i] = 0;
                self.pos_list_b.push(i as Pos);
            }
        }
        // init pos list and hash
        for i in 0..90 {
            self.key ^= HASH_KEYS[(board[i] + 7) as usize][i];
            if board[i] != 0 {
                self.bl10_items[i % 9] |= 1 << (i / 9);
                self.bl9_items[i / 9] |= 1 << (i % 9);
            }
            if board[i] > 0 && board[i] != R_KING {
                self.pos_pid_r[i] = self.pos_list_r.len() as Pid;
                self.pos_list_r.push(i as Pos);
            } else if board[i] < 0 && board[i] != B_KING {
                self.pos_pid_b[i] = self.pos_list_b.len() as Pid;
                self.pos_list_b.push(i as Pos);
            }
        }
        self.history.moves.reserve(256);
        self.history.captures.reserve(256);
        self.history.checkings.reserve(256);
        self.history.keys.reserve(256);

        self.eval_reset();
        for i in 0..90 {
            if board[i] != 0 {
                self.eval_add(if board[i] > 0 { R } else { B }, board[i], i as Pos);
            }
        }
    }
    pub fn get_pos_list(&self) -> Vec<Pos> {
        if self.team == R {
            self.pos_list_r.clone()
        } else {
            self.pos_list_b.clone()
        }
    }
    pub fn target_chk(&self, pos: Pos, g: GenType) -> bool {
        if pos >= 90 {
            return false;
        }
        let target = self.board[pos as usize];
        if target.abs() == R_KING {
            return false;
        }
        if g == CAPTURE {
            (target * self.team as i8) < 0
        } else if g == QUIET {
            target == 0
        } else {
            (target * self.team as i8) <= 0
        }
    }
    pub fn get_piece(&self, pos: Pos) -> PType {
        if pos >= 90 {
            return 0;
        }
        self.board[pos as usize]
    }
    pub fn get_bl10(&self, col: Pos) -> u16 {
        self.bl10_items[col as usize % 9]
    }
    pub fn get_bl9(&self, row: Pos) -> u16 {
        self.bl9_items[row as usize / 9]
    }
    pub fn face_king(&self) -> bool {
        if self.pos_list_r[0] % 9 == self.pos_list_b[0] % 9 {
            return matches!(
                self.get_bl10(self.pos_list_r[0] % 9),
                0b1000000001
                    | 0b1000000010
                    | 0b1000000100
                    | 0b0100000001
                    | 0b0100000010
                    | 0b0100000100
                    | 0b0010000001
                    | 0b0010000010
                    | 0b0010000100
            );
        }
        false
    }
    pub fn do_move(&mut self, mv: Move) {
        let beg = mv.beg as usize;
        let end = mv.end as usize;
        assert!(mv.into() && self.board[beg] * self.team > 0);
        assert!(self.board[end] * self.team <= 0 && self.board[end].abs() != R_KING);
        assert!(!self.pos_list_r.is_empty() && !self.pos_list_b.is_empty());

        self.history.moves.push(mv);
        self.history.captures.push(self.board[end]);
        self.history.keys.push(self.key);
        self.history.checkings.push(false);

        self.eval_slide(self.team, self.board[beg], mv.beg, mv.end);
        if self.board[end] != 0 {
            self.eval_remove(-self.team, self.board[end], mv.end);
        }

        self.key ^= HASH_KEYS[(self.board[beg] + 7) as usize][beg];
        self.key ^= HASH_KEYS[(self.board[end] + 7) as usize][end];
        self.key ^= HASH_KEYS[(self.board[beg] + 7) as usize][end];
        self.key ^= SIDE_KEY;

        if self.team == R {
            self.pos_list_r[self.pos_pid_r[beg] as usize] = mv.end;
            self.pos_pid_r.swap(beg, end);
            if self.board[end] != 0 {
                let pid = self.pos_pid_b[end];
                let last_pos = self.pos_list_b.pop().unwrap();
                self.pos_list_b[pid as usize] = last_pos;
                self.pos_pid_b[last_pos as usize] = pid;
                self.pos_pid_b[end] = 0;
            }
        } else {
            self.pos_list_b[self.pos_pid_b[beg] as usize] = mv.end;
            self.pos_pid_b.swap(beg, end);
            if self.board[end] != 0 {
                let pid = self.pos_pid_r[end];
                let last_pos = self.pos_list_r.pop().unwrap();
                self.pos_list_r[pid as usize] = last_pos;
                self.pos_pid_r[last_pos as usize] = pid;
                self.pos_pid_r[end] = 0;
            }
        }

        self.bl10_items[end % 9] |= 1 << (mv.end / 9);
        self.bl9_items[end / 9] |= 1 << (mv.end % 9);
        self.bl10_items[beg % 9] &= !(1 << (mv.beg / 9));
        self.bl9_items[beg / 9] &= !(1 << (mv.beg % 9));

        self.board[end] = self.board[beg];
        self.board[beg] = 0;
        self.team = -self.team;
    }
    pub fn undo_move(&mut self) {
        assert!(!self.history.moves.is_empty());
        let mv = self.history.moves.pop().unwrap();
        let beg = mv.beg as usize;
        let end = mv.end as usize;
        let captured = self.history.captures.pop().unwrap();
        let prev_key = self.history.keys.pop().unwrap();
        self.history.checkings.pop();

        self.eval_slide(-self.team, self.board[end], mv.end, mv.beg);
        if captured != 0 {
            self.eval_add(self.team, captured, mv.end);
        }
        self.key = prev_key;
        if self.team == R {
            self.pos_list_b[self.pos_pid_b[end] as usize] = mv.beg;
            self.pos_pid_b.swap(beg, end);
            if captured != 0 {
                self.pos_pid_r[end] = self.pos_list_r.len() as Pid;
                self.pos_list_r.push(mv.end);
            }
        } else {
            self.pos_list_r[self.pos_pid_r[end] as usize] = mv.beg;
            self.pos_pid_r.swap(beg, end);
            if captured != 0 {
                self.pos_pid_b[end] = self.pos_list_b.len() as Pid;
                self.pos_list_b.push(mv.end);
            }
        }

        self.bl10_items[end % 9] &= if captured == 0 {
            !(1 << (mv.end / 9))
        } else {
            0xFFF
        };
        self.bl9_items[end / 9] &= if captured == 0 {
            !(1 << (mv.end % 9))
        } else {
            0xFFF
        };
        self.bl10_items[beg % 9] |= 1 << (mv.beg / 9);
        self.bl9_items[beg / 9] |= 1 << (mv.beg % 9);

        self.board[beg] = self.board[end];
        self.board[end] = captured;
        self.team = -self.team;
    }
    pub fn do_null(&mut self) {
        self.key ^= SIDE_KEY;
        self.team = -self.team;
    }
    pub fn undo_null(&mut self) {
        self.key ^= SIDE_KEY;
        self.team = -self.team;
    }
    pub fn in_check(&self) -> bool {
        let pos = if self.team == R {
            self.pos_list_r[0]
        } else {
            self.pos_list_b[0]
        };
        assert!(pos % 9 >= 3 && pos % 9 <= 5);
        assert!((self.team == R && pos / 9 >= 7) || (self.team == B && pos / 9 <= 2));
        assert!(self.get_piece(pos).abs() == R_KING);
        // attacked by pawn
        if self.get_piece((pos as i8 - 9 * self.team) as Pos) * self.team == B_PAWN {
            return true;
        }
        if self.get_piece(pos - 1) * self.team == B_PAWN {
            return true;
        }
        if self.get_piece(pos + 1) * self.team == B_PAWN {
            return true;
        }
        // attacked by knight
        if self.get_piece(pos - 10) == 0 {
            return self.get_piece(pos - 19) * self.team == B_KNIGHT
                || self.get_piece(pos - 11) * self.team == B_KNIGHT;
        }
        if self.get_piece(pos + 10) == 0 {
            return self.get_piece(pos + 19) * self.team == B_KNIGHT
                || self.get_piece(pos + 11) * self.team == B_KNIGHT;
        }
        if self.get_piece(pos + 8) == 0 {
            return self.get_piece(pos + 17) * self.team == B_KNIGHT
                || self.get_piece(pos + 7) * self.team == B_KNIGHT;
        }
        if self.get_piece(pos - 8) == 0 {
            return self.get_piece(pos - 17) * self.team == B_KNIGHT
                || self.get_piece(pos - 7) * self.team == B_KNIGHT;
        }

        let (l, r) = rook9(self.get_bl9(pos), pos);
        let (top, bot) = rook10(self.get_bl10(pos), pos);
        if self.get_piece(l) * self.team == B_ROOK
            || self.get_piece(r) * self.team == B_ROOK
            || self.get_piece(top) * self.team == B_ROOK
            || self.get_piece(bot) * self.team == B_ROOK
        {
            return true;
        }
        let (l, r) = cannon9(self.get_bl9(pos), pos);
        let (top, bot) = cannon10(self.get_bl10(pos), pos);
        if l < 90 && self.get_piece(l) * self.team == B_CANNON {
            return true;
        }
        if r < 90 && self.get_piece(r) * self.team == B_CANNON {
            return true;
        }
        if top < 90 && self.get_piece(top) * self.team == B_CANNON {
            return true;
        }
        if bot < 90 && self.get_piece(bot) * self.team == B_CANNON {
            return true;
        }

        self.face_king()
    }
    pub fn legal_move(&self, mv: Move) -> bool {
        let p = self.get_piece(mv.beg);
        if self.team * self.get_piece(mv.end) > 0 || p * self.team <= 0 {
            return false;
        }
        if self.get_piece(mv.end).abs() == R_KING {
            return false;
        }
        let d = mv.end as i32 - mv.beg as i32;
        if p.abs() == R_KING {
            if d.abs() != 1 && d.abs() != 9 {
                return false;
            }
            if mv.end % 9 < 3 || mv.end % 9 > 5 {
                return false;
            }
            if p == R_KING && mv.end / 9 < 7 {
                return false;
            }
            if p == B_KING && mv.end / 9 > 2 {
                return false;
            }
        } else if p.abs() == R_ADVISOR {
            if d.abs() != 10 && d.abs() != 8 {
                return false;
            }
            if mv.end != 13
                && mv.end != 76
                && !(mv.end % 9 >= 3
                    && mv.end % 9 <= 5
                    && ((p == R_ADVISOR && mv.end / 9 >= 7) || (p == B_ADVISOR && mv.end / 9 <= 2)))
            {
                return false;
            }
            if p == R_ADVISOR && mv.end / 9 < 7 {
                return false;
            }
            if p == B_ADVISOR && mv.end / 9 > 2 {
                return false;
            }
        } else if p.abs() == R_BISHOP {
            if d == 20 {
                if self.get_piece(mv.beg + 10) != 0 {
                    return false;
                }
            } else if d == 16 {
                if self.get_piece(mv.beg + 8) != 0 {
                    return false;
                }
            } else if d == -20 {
                if self.get_piece(mv.beg - 10) != 0 {
                    return false;
                }
            } else if self.get_piece(mv.beg - 8) != 0 {
                return false;
            }
        } else if p.abs() == R_KNIGHT {
            if d == 17 || d == 15 {
                if self.get_piece(mv.beg + 9) != 0 {
                    return false;
                }
            } else if d == -17 || d == -15 {
                if self.get_piece(mv.beg - 9) != 0 {
                    return false;
                }
            } else if d == 10 || d == -6 {
                if self.get_piece(mv.beg + 1) != 0 {
                    return false;
                }
            } else if self.get_piece(mv.beg - 1) != 0 {
                return false;
            }
        } else if p.abs() == R_CANNON {
            if mv.beg % 9 != mv.end % 9 || mv.beg / 9 != mv.end / 9 {
                return false;
            }
            if -9 < d && d < 9 {
                let (l, r) = cannon9(self.get_bl9(mv.beg), mv.beg);
                let (l2, r2) = rook9(self.get_bl9(mv.end), mv.end);
                if !(l2 < mv.end && mv.end < r2) {
                    if mv.end != l && mv.end != r {
                        return false;
                    }
                }
            } else {
                let (u, d) = cannon10(self.get_bl10(mv.beg), mv.beg);
                let (u2, d2) = rook10(self.get_bl10(mv.end), mv.end);
                if !(u2 < mv.end && mv.end < d2) {
                    if mv.end != u && mv.end != d {
                        return false;
                    }
                }
            }
        } else if p.abs() == R_ROOK {
            if mv.beg % 9 != mv.end % 9 || mv.beg / 9 != mv.end / 9 {
                return false;
            }
            if -9 < d && d < 9 {
                let (l, r) = rook9(self.get_bl9(mv.beg), mv.beg);
                if !(l < mv.end && mv.end < r) {
                    return false;
                }
            } else {
                let (u, d) = rook10(self.get_bl10(mv.beg), mv.beg);
                if !(u < mv.end && mv.end < d) {
                    return false;
                }
            }
        }
        true
    }
    pub fn get_protector(&self, pos: Pos) -> Pos {
        let team = self.team;
        if self.get_piece((pos as i8 - 9 * team) as u8) * team == R_PAWN {
            return (pos as i8 - 9 * team) as u8;
        }
        if (pos / 9 < 5 && team == R) || (pos / 9 > 4 && team == B) {
            if self.get_piece(pos - 1) * team == R_PAWN {
                return pos - 1;
            }
            if self.get_piece(pos + 1) * team == R_PAWN {
                return pos + 1;
            }
        }

        let c1 = team == R && pos / 9 > 6 && pos % 9 > 2 && pos % 9 < 5;
        let c2 = team == B && pos / 9 < 3 && pos % 9 > 2 && pos % 9 < 5;
        if c1 || c2 {
            if self.get_piece(pos - 10) * team == R_ADVISOR {
                return pos - 10;
            }
            if self.get_piece(pos - 8) * team == R_ADVISOR {
                return pos - 8;
            }
            if self.get_piece(pos + 10) * team == R_ADVISOR {
                return pos + 10;
            }
            if self.get_piece(pos + 8) * team == R_ADVISOR {
                return pos + 8;
            }
        }

        if team == R && pos / 9 > 4 || team == B && pos / 9 < 5 {
            if self.get_piece(pos - 10) == 0 && self.get_piece(pos - 20) * team == R_BISHOP {
                return pos - 20;
            }
            if self.get_piece(pos - 8) == 0 && self.get_piece(pos - 16) * team == R_BISHOP {
                return pos - 16;
            }
            if self.get_piece(pos + 20) * team == R_BISHOP {
                return pos + 20;
            }
            if self.get_piece(pos + 16) * team == R_BISHOP {
                return pos + 16;
            }
        }

        if self.get_piece(pos - 10) == 0 {
            if self.get_piece(pos - 19) * team == R_KNIGHT {
                return pos - 19;
            }
            if self.get_piece(pos - 11) * team == R_KNIGHT {
                return pos - 11;
            }
        }
        if self.get_piece(pos + 10) == 0 {
            if self.get_piece(pos + 19) * team == R_KNIGHT {
                return pos + 19;
            }
            if self.get_piece(pos + 11) * team == R_KNIGHT {
                return pos + 11;
            }
        }
        if self.get_piece(pos - 8) == 0 {
            if self.get_piece(pos - 17) * team == R_KNIGHT {
                return pos - 17;
            }
            if self.get_piece(pos - 7) * team == R_KNIGHT {
                return pos - 7;
            }
        }
        if self.get_piece(pos + 8) == 0 {
            if self.get_piece(pos + 17) * team == R_KNIGHT {
                return pos + 17;
            }
            if self.get_piece(pos + 7) * team == R_KNIGHT {
                return pos + 7;
            }
        }
        let (l, r) = rook9(self.get_bl9(pos), pos);
        let (u, b) = rook10(self.get_bl10(pos), pos);
        let (l2, r2) = cannon9(self.get_bl9(pos), pos);
        let (u2, b2) = cannon10(self.get_bl10(pos), pos);
        if l2 < 90 && self.get_piece(l2) * team == R_CANNON {
            return l2;
        }
        if r2 < 90 && self.get_piece(r2) * team == R_CANNON {
            return r2;
        }
        if u2 < 90 && self.get_piece(u2) * team == R_CANNON {
            return u2;
        }
        if b2 < 90 && self.get_piece(b2) * team == R_CANNON {
            return b2;
        }
        if self.get_piece(l) * team == R_ROOK {
            return l;
        }
        if self.get_piece(r) * team == R_ROOK {
            return r;
        }
        if self.get_piece(u) * team == R_ROOK {
            return u;
        }
        if self.get_piece(b) * team == R_ROOK {
            return b;
        }

        if c1 || c2 {
            if self.get_piece(pos - 9) * team == R_KING {
                return pos - 9;
            }
            if self.get_piece(pos + 9) * team == R_KING {
                return pos + 9;
            }
            if self.get_piece(pos - 1) * team == R_KING {
                return pos - 1;
            }
            if self.get_piece(pos + 1) * team == R_KING {
                return pos + 1;
            }
        }
        INVALID_POS
    }
    pub fn is_repeat(&self) -> bool {
        let h = &self.history;
        let n = h.keys.len();
        if n < 4 || h.checkings.len() != n {
            return false;
        }
        for i in (0..n).rev() {
            if h.keys[i] == self.key {
                if n - i < 4 {
                    return false;
                }
                let mut j = i;
                while j < n {
                    if !h.checkings[j] {
                        return false;
                    }
                    j += 2;
                }
                return true;
            }
            if h.captures[i] != 0 {
                break;
            }
        }
        false
    }
}
