//! The rules: a 7×8 board of poppets, swap two neighbours to line up three or
//! more of a kind, matches pop, the rest fall, new ones drop in. A wake meter
//! drains all the time (faster and faster); pops refill it. Empty = asleep =
//! game over.
//!
//! Pure game state: no host calls. Input comes in as `swap(a, b)`, time as
//! `step(dt)`; sound and juice go out as `cues`. The board resolves in
//! discrete steps (swap → pop → fall → pop → … → ready) and each step lasts
//! a fixed time the host animates (`phase` and its timer, `fall_y`), so the
//! same seed and the same swaps at the same times give the same round.

use maimbrain::Rng;

// ---- Tuning knobs -----------------------------------------------------------
/// Board size in cells. Wider boards have more moves and longer cascades.
pub const COLS: usize = 7;
pub const ROWS: usize = 8;
/// Kinds of poppet in play at the start. Fewer kinds = more matches and cascades
/// (5 is generous; 6 is a lot harder to read).
pub const START_COLORS: u8 = 5;
/// The wake meter is 0…1. Drain per second at the start of a round.
pub const DRAIN_START: f32 = 0.042;
/// Drain added per second of play (the meter empties faster and faster).
pub const DRAIN_GROWTH: f32 = 0.0016;
/// The drain stops growing here (bar fractions per second).
pub const DRAIN_MAX: f32 = 0.3;
/// Meter refilled per poppet popped. Raise it and every round gets longer.
pub const GAIN_PIECE: f32 = 0.0175;
/// Each cascade step multiplies the refill by (1 + GAIN_CHAIN × step).
pub const GAIN_CHAIN: f32 = 0.5;
/// Refill for making a special (line, bomb, rainbow) and for a rock broken.
pub const GAIN_MADE: f32 = 0.04;
pub const GAIN_ROCK: f32 = 0.03;
/// Refill for popping a star poppet (from level 6).
pub const GAIN_STAR: f32 = 0.16;
/// Below this the meter is in danger (a cue, the tension music, faces nodding off).
pub const DROWSY: f32 = 0.25;
/// Faces start to droop below this.
pub const DROWSY_FROM: f32 = 0.45;
/// Saved from below this: "PHEW!".
pub const PHEW_BELOW: f32 = 0.1;
/// Seconds per level. Every level brings something new (`news`).
pub const LEVEL_TIME: f32 = 15.0;
/// Animation timings the sim waits for (seconds; the fall is gravity in cells/s²).
pub const SWAP_TIME: f32 = 0.13;
pub const POP_TIME: f32 = 0.2;
pub const FALL_G: f32 = 80.0;
pub const LAND_SETTLE: f32 = 0.1;
pub const SHUFFLE_TIME: f32 = 0.75;
/// Between the meter running out and the game-over card.
pub const ENDING: f32 = 1.7;
/// A blast reaches cells this much later per cell of distance (a wave).
pub const BLAST_STAGGER: f32 = 0.035;
/// A swipe made while the board is still settling is kept this long.
pub const QUEUE_TIME: f32 = 0.3;
/// Rocks: chance per new poppet once rocks are on, and the most on the board.
pub const ROCK_CHANCE: f32 = 0.05;
pub const ROCKS_MAX: u32 = 4;
/// Chance a new poppet is a star once stars are on.
pub const STAR_CHANCE: f32 = 0.03;
/// Points per poppet popped, times the cascade step.
pub const POINTS_PIECE: u32 = 10;

// ---- Pieces -----------------------------------------------------------------

/// What sits in a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A poppet of one of the colors (0…6, see `look::PIECES`).
    Color(u8),
    /// Made from 5 in a row: swap it with any poppet to pop all of that color.
    Rainbow,
    /// A blocker: can't be moved or matched; a match next to it (or a blast) breaks it.
    Rock,
}

/// A power a colored poppet can carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Special {
    None,
    /// Made from 4 in a row: pops its whole row (H) or column (V) when popped.
    LineH,
    LineV,
    /// Made from an L or T: pops the 3×3 around it.
    Bomb,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    pub kind: Kind,
    pub special: Special,
    /// A star poppet: a big refill when popped.
    pub star: bool,
    /// Unique per piece (faces blink on their own clock).
    pub id: u32,
    /// Where it was drawn when the current fall or shuffle began (in cells).
    pub from_x: f32,
    pub from_y: f32,
}

impl Piece {
    /// The color it matches as, if any.
    pub fn color(&self) -> Option<u8> {
        match self.kind {
            Kind::Color(c) => Some(c),
            _ => None,
        }
    }
    pub fn is_special(&self) -> bool {
        self.special != Special::None || self.kind == Kind::Rainbow
    }
}

/// (column, row); row 0 is the top.
pub type Cell = (usize, usize);
/// A swap: the piece at `.0` moves to `.1`.
pub type Move = (Cell, Cell);

pub fn adjacent(a: Cell, b: Cell) -> bool {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1) == 1
}

fn dist(a: Cell, b: Cell) -> f32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1)) as f32
}

#[derive(Clone, Debug)]
pub struct Board {
    pub g: [[Option<Piece>; COLS]; ROWS],
}

impl Board {
    pub fn get(&self, c: Cell) -> Option<Piece> {
        self.g[c.1][c.0]
    }
    fn set(&mut self, c: Cell, p: Option<Piece>) {
        self.g[c.1][c.0] = p;
    }
    fn color(&self, c: usize, r: usize) -> Option<u8> {
        self.g[r][c].and_then(|p| p.color())
    }
    pub fn swap(&mut self, a: Cell, b: Cell) {
        let t = self.get(a);
        self.set(a, self.get(b));
        self.set(b, t);
    }
    pub fn cells() -> impl Iterator<Item = Cell> {
        (0..ROWS).flat_map(|r| (0..COLS).map(move |c| (c, r)))
    }
    pub fn count(&self, f: impl Fn(&Piece) -> bool) -> u32 {
        Board::cells().filter(|&c| self.get(c).is_some_and(|p| f(&p))).count() as u32
    }
}

// ---- Matching ---------------------------------------------------------------

/// Three or more of one color touching in rows and/or columns.
#[derive(Clone, Debug)]
pub struct Group {
    pub cells: Vec<Cell>,
    pub color: u8,
    pub longest: usize,
    pub longest_horiz: bool,
    /// The middle of the longest run.
    pub mid: Cell,
    /// A cell in both a row run and a column run (L, T, +).
    pub cross: Option<Cell>,
}

/// What a group makes: 5 in a row a rainbow, an L/T a bomb, 4 a line blaster.
pub fn make_of(g: &Group) -> Option<(Kind, Special)> {
    if g.longest >= 5 {
        Some((Kind::Rainbow, Special::None))
    } else if g.cross.is_some() {
        Some((Kind::Color(g.color), Special::Bomb))
    } else if g.longest == 4 {
        Some((Kind::Color(g.color), if g.longest_horiz { Special::LineH } else { Special::LineV }))
    } else {
        None
    }
}

struct Run {
    cells: Vec<Cell>,
    horiz: bool,
}

fn runs(b: &Board) -> Vec<Run> {
    let mut out = Vec::new();
    for r in 0..ROWS {
        let mut c = 0;
        while c < COLS {
            let col = b.color(c, r);
            let mut e = c + 1;
            if col.is_some() {
                while e < COLS && b.color(e, r) == col {
                    e += 1;
                }
                if e - c >= 3 {
                    out.push(Run { cells: (c..e).map(|x| (x, r)).collect(), horiz: true });
                }
            }
            c = e;
        }
    }
    for c in 0..COLS {
        let mut r = 0;
        while r < ROWS {
            let col = b.color(c, r);
            let mut e = r + 1;
            if col.is_some() {
                while e < ROWS && b.color(c, e) == col {
                    e += 1;
                }
                if e - r >= 3 {
                    out.push(Run { cells: (r..e).map(|y| (c, y)).collect(), horiz: false });
                }
            }
            r = e;
        }
    }
    out
}

/// Every match on the board, runs that share a cell merged into one group.
pub fn find_groups(b: &Board) -> Vec<Group> {
    let rs = runs(b);
    let n = rs.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn root(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    for i in 0..n {
        for j in i + 1..n {
            if rs[i].horiz != rs[j].horiz && rs[i].cells.iter().any(|c| rs[j].cells.contains(c)) {
                let (a, bb) = (root(&mut parent, i), root(&mut parent, j));
                parent[a] = bb;
            }
        }
    }
    let mut out = Vec::new();
    for i in 0..n {
        if root(&mut parent, i) != i {
            continue;
        }
        let members: Vec<usize> = (0..n).filter(|&j| root(&mut parent, j) == i).collect();
        let mut cells: Vec<Cell> = Vec::new();
        let (mut longest, mut horiz, mut mid, mut cross) = (0, true, (0, 0), None);
        for &j in &members {
            let run = &rs[j];
            for &c in &run.cells {
                if cells.contains(&c) {
                    cross = Some(c);
                } else {
                    cells.push(c);
                }
            }
            if run.cells.len() > longest {
                longest = run.cells.len();
                horiz = run.horiz;
                mid = run.cells[run.cells.len() / 2];
            }
        }
        let color = b.color(cells[0].0, cells[0].1).unwrap_or(0);
        out.push(Group { cells, color, longest, longest_horiz: horiz, mid, cross });
    }
    out
}

/// Does the piece at `c` sit in a line of 3?
fn match_at(b: &Board, c: Cell) -> bool {
    let Some(col) = b.color(c.0, c.1) else { return false };
    let same = |x: i32, y: i32| x >= 0 && y >= 0 && (x as usize) < COLS && (y as usize) < ROWS && b.color(x as usize, y as usize) == Some(col);
    let (x, y) = (c.0 as i32, c.1 as i32);
    let line = |dx: i32, dy: i32| {
        let mut n = 1;
        let mut k = 1;
        while same(x + dx * k, y + dy * k) {
            n += 1;
            k += 1;
        }
        k = 1;
        while same(x - dx * k, y - dy * k) {
            n += 1;
            k += 1;
        }
        n
    };
    line(1, 0) >= 3 || line(0, 1) >= 3
}

/// Whether swapping a and b does something (a match, a rainbow, two specials).
pub fn swap_valid(b: &Board, a: Cell, c: Cell) -> bool {
    let (Some(pa), Some(pc)) = (b.get(a), b.get(c)) else { return false };
    if !adjacent(a, c) || pa.kind == Kind::Rock || pc.kind == Kind::Rock {
        return false;
    }
    if pa.kind == Kind::Rainbow || pc.kind == Kind::Rainbow || (pa.is_special() && pc.is_special()) {
        return true;
    }
    let mut t = b.clone();
    t.swap(a, c);
    match_at(&t, a) || match_at(&t, c)
}

/// Every useful swap on the board (each pair once).
pub fn moves(b: &Board) -> Vec<Move> {
    let mut out = Vec::new();
    for (c, r) in Board::cells() {
        for nb in [(c + 1, r), (c, r + 1)] {
            if nb.0 < COLS && nb.1 < ROWS && swap_valid(b, (c, r), nb) {
                out.push(((c, r), nb));
            }
        }
    }
    out
}

// ---- Resolving --------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlastKind {
    Row,
    Col,
    /// A square, this many cells out from the centre.
    Area(u8),
    /// Rows and columns through the centre, this many cells either side.
    Cross(u8),
    /// A rainbow zapping every target.
    Zap,
    /// Two rainbows: the whole board.
    Board,
}

/// A special going off, for the host to draw (beams, rings, zaps).
#[derive(Clone, Debug)]
pub struct Blast {
    pub kind: BlastKind,
    pub at: Cell,
    pub delay: f32,
    pub color: Option<u8>,
    pub targets: Vec<Cell>,
}

/// A piece leaving the board this step.
#[derive(Clone, Copy, Debug)]
pub struct Popping {
    pub at: Cell,
    pub piece: Piece,
    /// When it pops, from the start of the step.
    pub delay: f32,
    /// It flies into this cell (where its group made a special).
    pub to: Option<Cell>,
}

/// One group's pop, for the score popup.
#[derive(Clone, Copy, Debug)]
pub struct GroupPop {
    pub x: f32,
    pub y: f32,
    pub points: u32,
}

/// What one resolution step did.
#[derive(Clone, Debug, Default)]
pub struct Step {
    pub popping: Vec<Popping>,
    pub blasts: Vec<Blast>,
    pub made: Vec<(Cell, Piece)>,
    pub groups: Vec<GroupPop>,
    pub pieces: u32,
    pub rocks: u32,
    pub stars: u32,
    pub fires: u32,
    pub combo: bool,
    pub dur: f32,
}

/// The cells a special covers when it goes off at `at`.
fn area(at: Cell, kind: BlastKind) -> Vec<Cell> {
    let (c, r) = (at.0 as i32, at.1 as i32);
    Board::cells()
        .filter(|&(x, y)| {
            let (dx, dy) = ((x as i32 - c).abs(), (y as i32 - r).abs());
            match kind {
                BlastKind::Row => dy == 0,
                BlastKind::Col => dx == 0,
                BlastKind::Area(k) => dx <= k as i32 && dy <= k as i32,
                BlastKind::Cross(k) => dx <= k as i32 || dy <= k as i32,
                BlastKind::Zap | BlastKind::Board => true,
            }
        })
        .collect()
}

fn blast_of(s: Special) -> BlastKind {
    match s {
        Special::LineH => BlastKind::Row,
        Special::LineV => BlastKind::Col,
        _ => BlastKind::Area(1),
    }
}

/// The color with the most poppets (rainbows hit by a blast take it).
fn commonest(b: &Board, skip: &[[bool; COLS]; ROWS]) -> u8 {
    let mut n = [0u32; 8];
    for (c, r) in Board::cells() {
        if !skip[r][c]
            && let Some(k) = b.color(c, r)
        {
            n[k as usize] += 1;
        }
    }
    (0..8).max_by_key(|&k| (n[k], 8 - k)).unwrap_or(0) as u8
}

/// Pops whatever the board (just swapped `swap`, if a player move) has
/// matched, fires the specials caught in it, makes new specials, and
/// removes the popped pieces (leaving holes for `collapse`).
pub fn resolve(b: &mut Board, rng: &mut Rng, next_id: &mut u32, swap: Option<Move>, chain: u32) -> Step {
    let mut st = Step::default();
    let mut hit = [[false; COLS]; ROWS];
    let mut delay = [[0.0f32; COLS]; ROWS];
    let mut fly: [[Option<Cell>; COLS]; ROWS] = [[None; COLS]; ROWS];
    let mut done = [[false; COLS]; ROWS]; // a special that fired (or was spent in a combo)
    let mut protect = [[false; COLS]; ROWS];
    let mut queue: Vec<(Cell, f32)> = Vec::new();
    let hit_cell = |hit: &mut [[bool; COLS]; ROWS], delay: &mut [[f32; COLS]; ROWS], c: Cell, d: f32| {
        if hit[c.1][c.0] {
            delay[c.1][c.0] = delay[c.1][c.0].min(d);
        } else {
            hit[c.1][c.0] = true;
            delay[c.1][c.0] = d;
        }
    };

    // Swapping a rainbow or two specials: a combo instead of a match.
    if let Some((a, to)) = swap {
        let (pm, po) = (b.get(to).unwrap(), b.get(a).unwrap()); // the moved piece is now at `to`
        if pm.kind == Kind::Rainbow || po.kind == Kind::Rainbow {
            st.combo = true;
            let (rc, oc, o) = if pm.kind == Kind::Rainbow { (to, a, po) } else { (a, to, pm) };
            hit_cell(&mut hit, &mut delay, rc, 0.0);
            done[rc.1][rc.0] = true;
            if o.kind == Kind::Rainbow {
                done[oc.1][oc.0] = true;
                for c in Board::cells() {
                    hit_cell(&mut hit, &mut delay, c, dist(c, to) * BLAST_STAGGER);
                }
                st.blasts.push(Blast { kind: BlastKind::Board, at: to, delay: 0.0, color: None, targets: Vec::new() });
                st.fires += 2;
            } else if let Some(k) = o.color() {
                let mut targets = Vec::new();
                for c in Board::cells() {
                    let Some(mut p) = b.get(c) else { continue };
                    if p.color() != Some(k) {
                        continue;
                    }
                    let d = 0.08 + dist(c, rc) * BLAST_STAGGER;
                    if o.special != Special::None && p.special == Special::None {
                        // Every poppet of that color becomes the special, then they all go off.
                        p.special = match o.special {
                            Special::Bomb => Special::Bomb,
                            _ => if rng.f32() < 0.5 { Special::LineH } else { Special::LineV },
                        };
                        b.set(c, Some(p));
                    }
                    hit_cell(&mut hit, &mut delay, c, d);
                    targets.push(c);
                }
                st.blasts.push(Blast { kind: BlastKind::Zap, at: rc, delay: 0.0, color: Some(k), targets });
                st.fires += 1;
            }
        } else if pm.is_special() && po.is_special() {
            st.combo = true;
            done[a.1][a.0] = true;
            done[to.1][to.0] = true;
            hit_cell(&mut hit, &mut delay, a, 0.0);
            hit_cell(&mut hit, &mut delay, to, 0.0);
            let bombs = [pm, po].iter().filter(|p| p.special == Special::Bomb).count();
            let kind = match bombs {
                0 => BlastKind::Cross(0),
                1 => BlastKind::Cross(1),
                _ => BlastKind::Area(2),
            };
            let targets = area(to, kind);
            for &c in &targets {
                hit_cell(&mut hit, &mut delay, c, dist(c, to) * BLAST_STAGGER);
            }
            st.blasts.push(Blast { kind, at: to, delay: 0.0, color: pm.color(), targets: Vec::new() });
            st.fires += 2;
        }
    }

    // Matches, and the specials they make.
    if !st.combo {
        for g in find_groups(b) {
            let make = make_of(&g);
            let mut at = None;
            if let Some((kind, special)) = make {
                let cell = match swap {
                    Some((_, to)) if g.cells.contains(&to) => to,
                    Some((a, _)) if g.cells.contains(&a) => a,
                    _ => g.cross.unwrap_or(g.mid),
                };
                if !protect[cell.1][cell.0] {
                    protect[cell.1][cell.0] = true;
                    // A special already there goes off before it's replaced.
                    if b.get(cell).is_some_and(|p| p.is_special()) {
                        queue.push((cell, 0.0));
                    }
                    let id = *next_id;
                    *next_id += 1;
                    let piece = Piece { kind, special, star: false, id, from_x: cell.0 as f32, from_y: cell.1 as f32 };
                    st.made.push((cell, piece));
                    at = Some(cell);
                }
            }
            let (mut sx, mut sy) = (0.0, 0.0);
            for &c in &g.cells {
                sx += c.0 as f32;
                sy += c.1 as f32;
                if Some(c) != at {
                    hit_cell(&mut hit, &mut delay, c, 0.0);
                    fly[c.1][c.0] = at;
                }
                // Rocks next to a match crack.
                for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                    let (x, y) = (c.0 as i32 + dx, c.1 as i32 + dy);
                    if x >= 0 && y >= 0 && (x as usize) < COLS && (y as usize) < ROWS && b.get((x as usize, y as usize)).is_some_and(|p| p.kind == Kind::Rock) {
                        hit_cell(&mut hit, &mut delay, (x as usize, y as usize), 0.05);
                    }
                }
            }
            let n = g.cells.len() as f32;
            st.groups.push(GroupPop { x: sx / n, y: sy / n, points: g.cells.len() as u32 * POINTS_PIECE * chain });
        }
    }

    // Specials caught in the pop go off, and set off others: a chain reaction.
    for c in Board::cells() {
        if hit[c.1][c.0] && !done[c.1][c.0] && !protect[c.1][c.0] && b.get(c).is_some_and(|p| p.is_special()) {
            queue.push((c, delay[c.1][c.0]));
        }
    }
    let mut qi = 0;
    while qi < queue.len() {
        let (at, t0) = queue[qi];
        qi += 1;
        if done[at.1][at.0] {
            continue;
        }
        done[at.1][at.0] = true;
        let Some(p) = b.get(at) else { continue };
        let (kind, targets, color) = if p.kind == Kind::Rainbow {
            let k = commonest(b, &hit);
            let t: Vec<Cell> = Board::cells().filter(|&c| b.color(c.0, c.1) == Some(k)).collect();
            (BlastKind::Zap, t, Some(k))
        } else {
            let k = blast_of(p.special);
            (k, area(at, k), p.color())
        };
        for &c in &targets {
            if protect[c.1][c.0] || c == at {
                continue;
            }
            let d = t0 + dist(c, at) * BLAST_STAGGER;
            let fresh = !hit[c.1][c.0];
            hit_cell(&mut hit, &mut delay, c, d);
            if fresh && !done[c.1][c.0] && b.get(c).is_some_and(|q| q.is_special()) {
                queue.push((c, d));
            }
        }
        st.fires += 1;
        st.blasts.push(Blast { kind, at, delay: t0, color, targets: if kind == BlastKind::Zap { targets } else { Vec::new() } });
    }

    // Take the popped pieces off the board; put the new specials on it.
    let mut longest = 0.0f32;
    for c in Board::cells() {
        if !hit[c.1][c.0] || protect[c.1][c.0] {
            continue;
        }
        let Some(p) = b.get(c) else { continue };
        let d = delay[c.1][c.0];
        longest = longest.max(d);
        st.popping.push(Popping { at: c, piece: p, delay: d, to: fly[c.1][c.0] });
        match p.kind {
            Kind::Rock => st.rocks += 1,
            _ => st.pieces += 1,
        }
        if p.star {
            st.stars += 1;
        }
        b.set(c, None);
    }
    for &(c, p) in &st.made {
        b.set(c, Some(p));
    }
    st.dur = POP_TIME + longest;
    st
}

/// Seconds a piece takes to fall `d` cells.
pub fn fall_time(d: f32) -> f32 {
    if d <= 0.0 { 0.0 } else { (2.0 * d / FALL_G).sqrt() }
}

/// Where a piece falling from row `from` to row `to` is, `t` seconds into
/// the fall, and how long it has been down (negative while still falling).
pub fn fall_y(from: f32, to: f32, t: f32) -> (f32, f32) {
    let ft = fall_time(to - from);
    if t >= ft { (to, t - ft) } else { (from + 0.5 * FALL_G * t * t, t - ft) }
}

// ---- The round --------------------------------------------------------------

/// Where the board is in its turn. The host animates each phase from its timer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// Waiting for a swap.
    Ready,
    /// Two pieces sliding past each other. `valid` ones pop next; others bounce back.
    Swap { a: Cell, b: Cell, t: f32, valid: bool },
    Bounce { a: Cell, b: Cell, t: f32 },
    /// `last.popping` popping (each after its delay).
    Pop { t: f32, dur: f32 },
    /// Everything falling into place (`fall_y` from each piece's `from_y`).
    Fall { t: f32, dur: f32 },
    /// No moves left: the pieces fly to new places (from `from_x`, `from_y`).
    Shuffle { t: f32 },
    /// The meter ran out: everyone falls asleep, then `over`.
    Asleep { t: f32 },
}

/// The new thing each level brings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum News {
    Faster,
    NewColor,
    Rocks,
    Stars,
    MoreRocks,
}

/// What a level brings when it arrives (level 1 is the start).
pub fn news(level: u32) -> News {
    match level {
        3 => News::NewColor,
        4 => News::Rocks,
        6 => News::Stars,
        8 => News::MoreRocks,
        _ => News::Faster,
    }
}

/// Moments for the host to play sounds, haptics and juice for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cue {
    Swap,
    Bounce,
    /// A resolution step began (`last` holds what pops); `chain` 1 = the move itself, 2+ = cascades.
    Pop { chain: u32, pieces: u32 },
    Made { at: Cell, kind: Kind, special: Special },
    Fire { kind: BlastKind },
    Combo,
    Rocks { n: u32 },
    Star { at: Cell },
    Cascade { chain: u32 },
    Level { level: u32, news: News },
    Shuffle,
    /// The meter fell into the danger zone.
    Drowsy,
    /// Saved from almost empty.
    Phew,
    /// The meter ran out.
    Asleep,
}

/// How good a swap looks right now (the first step only: what a person sees).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Preview {
    pub valid: bool,
    pub pieces: u32,
    pub made: u32,
    pub fires: u32,
}

impl Preview {
    pub fn value(&self) -> f32 {
        if !self.valid { -1.0 } else { self.pieces as f32 + 3.0 * self.made as f32 + 1.5 * self.fires as f32 }
    }
}

#[derive(Clone)]
pub struct Sim {
    pub board: Board,
    pub rng: Rng,
    pub phase: Phase,
    /// Seconds of play.
    pub t: f32,
    pub level: u32,
    /// 0…1. Empty = asleep.
    pub energy: f32,
    pub score: u32,
    /// The cascade step of the current move (0 while ready).
    pub chain: u32,
    pub best_chain: u32,
    pub colors: u8,
    pub rocks_on: bool,
    pub rocks_max: u32,
    pub rocks_due: u32,
    pub stars_on: bool,
    /// The last resolution step (what's popping now).
    pub last: Step,
    /// Useful swaps while ready; `hint` is the best-looking one.
    pub moves: Vec<Move>,
    pub hint: Option<Move>,
    /// Times the board has come to rest (a new turn).
    pub settles: u32,
    pub swaps: u32,
    pub cues: Vec<Cue>,
    pub over: bool,
    /// Title-card mode: no drain, no levels.
    pub demo: bool,
    queued: Option<(Cell, Cell, f32)>,
    next_id: u32,
}

impl Sim {
    pub fn new(seed: u64, demo: bool) -> Sim {
        let mut s = Sim {
            board: Board { g: [[None; COLS]; ROWS] },
            rng: Rng::new(seed),
            phase: Phase::Ready,
            t: 0.0,
            level: 1,
            energy: 1.0,
            score: 0,
            chain: 0,
            best_chain: 0,
            colors: START_COLORS,
            rocks_on: false,
            rocks_max: ROCKS_MAX,
            rocks_due: 0,
            stars_on: false,
            last: Step::default(),
            moves: Vec::new(),
            hint: None,
            settles: 0,
            swaps: 0,
            cues: Vec::new(),
            over: false,
            demo,
            queued: None,
            next_id: 1,
        };
        s.fill();
        s.settle();
        s
    }

    fn piece(&mut self, kind: Kind) -> Piece {
        let id = self.next_id;
        self.next_id += 1;
        Piece { kind, special: Special::None, star: false, id, from_x: 0.0, from_y: 0.0 }
    }

    /// A fresh board with no matches and at least one move.
    fn fill(&mut self) {
        loop {
            for (c, r) in Board::cells() {
                let mut k = (self.rng.next_u32() % self.colors as u32) as u8;
                for _ in 0..self.colors {
                    let left = c >= 2 && self.board.color(c - 1, r) == Some(k) && self.board.color(c - 2, r) == Some(k);
                    let up = r >= 2 && self.board.color(c, r - 1) == Some(k) && self.board.color(c, r - 2) == Some(k);
                    if !left && !up {
                        break;
                    }
                    k = (k + 1) % self.colors;
                }
                let mut p = self.piece(Kind::Color(k));
                p.from_x = c as f32;
                p.from_y = r as f32;
                self.board.set((c, r), Some(p));
            }
            if !moves(&self.board).is_empty() {
                break;
            }
        }
    }

    /// A new poppet for the top of a column.
    fn spawn(&mut self) -> Piece {
        let rocks = self.board.count(|p| p.kind == Kind::Rock);
        if self.rocks_on && rocks < self.rocks_max && (self.rocks_due > 0 || self.rng.f32() < ROCK_CHANCE) {
            self.rocks_due = self.rocks_due.saturating_sub(1);
            return self.piece(Kind::Rock);
        }
        let k = (self.rng.next_u32() % self.colors as u32) as u8;
        let mut p = self.piece(Kind::Color(k));
        p.star = self.stars_on && self.rng.f32() < STAR_CHANCE;
        p
    }

    /// The meter's drain right now, per second.
    pub fn drain(&self) -> f32 {
        (DRAIN_START + DRAIN_GROWTH * self.t).min(DRAIN_MAX)
    }

    pub fn ready(&self) -> bool {
        self.phase == Phase::Ready
    }

    /// The player swaps the piece at `a` toward `b`. False if that's not a
    /// swap at all (not neighbours, a rock, a hole, the round is over). A
    /// swipe while the board is settling is kept for a moment.
    pub fn swap(&mut self, a: Cell, b: Cell) -> bool {
        if self.over || matches!(self.phase, Phase::Asleep { .. }) || a.0 >= COLS || a.1 >= ROWS || b.0 >= COLS || b.1 >= ROWS || !adjacent(a, b) {
            return false;
        }
        let movable = |p: Option<Piece>| p.is_some_and(|p| p.kind != Kind::Rock);
        if !self.ready() {
            self.queued = Some((a, b, QUEUE_TIME));
            return true;
        }
        if !movable(self.board.get(a)) || !movable(self.board.get(b)) {
            return false;
        }
        let valid = swap_valid(&self.board, a, b);
        self.phase = Phase::Swap { a, b, t: 0.0, valid };
        self.swaps += 1;
        self.cues.push(Cue::Swap);
        true
    }

    /// What swapping `m` would pop in its first step (no peeking at refills).
    pub fn preview(&self, m: Move) -> Preview {
        if !swap_valid(&self.board, m.0, m.1) {
            return Preview::default();
        }
        let mut b = self.board.clone();
        b.swap(m.0, m.1);
        let (mut rng, mut id) = (self.rng.clone(), self.next_id);
        let s = resolve(&mut b, &mut rng, &mut id, Some(m), 1);
        Preview { valid: true, pieces: s.pieces + s.rocks, made: s.made.len() as u32, fires: s.fires }
    }

    pub fn step(&mut self, dt: f32) {
        if self.over {
            return;
        }
        let draining = matches!(self.phase, Phase::Ready | Phase::Swap { .. } | Phase::Bounce { .. });
        if !self.demo && !matches!(self.phase, Phase::Asleep { .. }) {
            self.t += dt;
            if draining {
                let before = self.energy;
                self.energy = (self.energy - self.drain() * dt).max(0.0);
                if before >= DROWSY && self.energy < DROWSY {
                    self.cues.push(Cue::Drowsy);
                }
            }
            let level = 1 + (self.t / LEVEL_TIME) as u32;
            while self.level < level {
                self.level += 1;
                self.level_up();
            }
        }
        if let Some(q) = &mut self.queued {
            q.2 -= dt;
            if q.2 <= 0.0 {
                self.queued = None;
            }
        }
        match self.phase {
            Phase::Ready => {
                if self.energy <= 0.0 && !self.demo {
                    self.phase = Phase::Asleep { t: 0.0 };
                    self.cues.push(Cue::Asleep);
                } else if let Some((a, b, _)) = self.queued.take() {
                    self.swap(a, b);
                }
            }
            Phase::Swap { a, b, t, valid } => {
                let t = t + dt;
                if t < SWAP_TIME {
                    self.phase = Phase::Swap { a, b, t, valid };
                } else if valid {
                    self.board.swap(a, b);
                    self.resolve(Some((a, b)));
                } else {
                    self.phase = Phase::Bounce { a, b, t: 0.0 };
                    self.cues.push(Cue::Bounce);
                }
            }
            Phase::Bounce { a, b, t } => {
                let t = t + dt;
                self.phase = if t < SWAP_TIME { Phase::Bounce { a, b, t } } else { Phase::Ready };
            }
            Phase::Pop { t, dur } => {
                let t = t + dt;
                if t < dur {
                    self.phase = Phase::Pop { t, dur };
                } else {
                    let dur = self.collapse();
                    self.phase = Phase::Fall { t: 0.0, dur };
                }
            }
            Phase::Fall { t, dur } => {
                let t = t + dt;
                if t < dur {
                    self.phase = Phase::Fall { t, dur };
                } else if !find_groups(&self.board).is_empty() {
                    self.resolve(None);
                } else {
                    self.settle();
                }
            }
            Phase::Shuffle { t } => {
                let t = t + dt;
                self.phase = if t < SHUFFLE_TIME { Phase::Shuffle { t } } else { Phase::Ready };
            }
            Phase::Asleep { t } => {
                let t = t + dt;
                self.phase = Phase::Asleep { t };
                if t >= ENDING {
                    self.over = true;
                }
            }
        }
    }

    fn level_up(&mut self) {
        let n = news(self.level);
        match n {
            News::NewColor => self.colors = (self.colors + 1).min(6),
            News::Rocks => {
                self.rocks_on = true;
                self.rocks_due += 3;
            }
            News::Stars => self.stars_on = true,
            News::MoreRocks => {
                self.rocks_max += 3;
                self.rocks_due += 3;
            }
            News::Faster => {}
        }
        self.cues.push(Cue::Level { level: self.level, news: n });
    }

    /// One resolution step: pop, score, refill the meter.
    fn resolve(&mut self, swap: Option<Move>) {
        self.chain += 1;
        self.best_chain = self.best_chain.max(self.chain);
        let st = resolve(&mut self.board, &mut self.rng, &mut self.next_id, swap, self.chain);
        let chain = self.chain;
        self.score += (st.pieces + 2 * st.rocks) * POINTS_PIECE * chain + st.made.len() as u32 * 30 + st.stars * 100 + if st.combo { 200 } else { 0 };
        let before = self.energy;
        let gain = (st.pieces as f32 * GAIN_PIECE) * (1.0 + GAIN_CHAIN * (chain - 1) as f32)
            + st.made.len() as f32 * GAIN_MADE
            + st.rocks as f32 * GAIN_ROCK
            + st.stars as f32 * GAIN_STAR;
        if !self.demo {
            self.energy = (self.energy + gain).min(1.0);
        }
        self.cues.push(Cue::Pop { chain, pieces: st.pieces });
        if chain >= 2 {
            self.cues.push(Cue::Cascade { chain });
        }
        if st.combo {
            self.cues.push(Cue::Combo);
        }
        for b in &st.blasts {
            self.cues.push(Cue::Fire { kind: b.kind });
        }
        for &(at, p) in &st.made {
            self.cues.push(Cue::Made { at, kind: p.kind, special: p.special });
        }
        if st.rocks > 0 {
            self.cues.push(Cue::Rocks { n: st.rocks });
        }
        for p in &st.popping {
            if p.piece.star {
                self.cues.push(Cue::Star { at: p.at });
            }
        }
        if before < PHEW_BELOW && self.energy >= PHEW_BELOW + 0.08 {
            self.cues.push(Cue::Phew);
        }
        self.phase = Phase::Pop { t: 0.0, dur: st.dur };
        self.last = st;
    }

    /// Everything falls into the holes; new poppets drop in from the top.
    /// Returns how long the fall takes.
    fn collapse(&mut self) -> f32 {
        let mut longest = 0.0f32;
        for c in 0..COLS {
            let mut write = ROWS;
            for r in (0..ROWS).rev() {
                if let Some(mut p) = self.board.g[r][c].take() {
                    write -= 1;
                    p.from_x = c as f32;
                    p.from_y = r as f32;
                    longest = longest.max(fall_time((write - r) as f32));
                    self.board.g[write][c] = Some(p);
                }
            }
            let empty = write;
            for r in 0..empty {
                let mut p = self.spawn();
                p.from_x = c as f32;
                p.from_y = r as f32 - empty as f32 - 0.3;
                longest = longest.max(fall_time(empty as f32 + 0.3));
                self.board.g[r][c] = Some(p);
            }
        }
        longest + LAND_SETTLE
    }

    /// The board has come to rest: a new turn (or a shuffle if it's stuck).
    pub(crate) fn settle(&mut self) {
        self.chain = 0;
        self.moves = moves(&self.board);
        if self.moves.is_empty() {
            self.shuffle();
            self.moves = moves(&self.board);
            self.phase = Phase::Shuffle { t: 0.0 };
            self.cues.push(Cue::Shuffle);
        } else {
            self.phase = Phase::Ready;
        }
        let values: Vec<f32> = self.moves.iter().map(|&m| self.preview(m).value()).collect();
        self.hint = (0..self.moves.len()).max_by(|&i, &j| values[i].total_cmp(&values[j]).then(j.cmp(&i))).map(|i| self.moves[i]);
        self.settles += 1;
    }

    /// Moves the poppets around (rocks stay) until there's a move and no match.
    fn shuffle(&mut self) {
        let cells: Vec<Cell> = Board::cells().filter(|&c| self.board.get(c).is_some_and(|p| p.kind != Kind::Rock)).collect();
        let mut pieces: Vec<Piece> = cells
            .iter()
            .map(|&c| {
                let mut p = self.board.get(c).unwrap();
                p.from_x = c.0 as f32;
                p.from_y = c.1 as f32;
                p
            })
            .collect();
        for attempt in 0..200 {
            for i in (1..pieces.len()).rev() {
                let j = (self.rng.next_u32() as usize) % (i + 1);
                pieces.swap(i, j);
            }
            if attempt >= 100 {
                // Stuck (rocks everywhere): new colors too.
                for p in pieces.iter_mut() {
                    if p.special == Special::None && p.kind != Kind::Rainbow {
                        p.kind = Kind::Color((self.rng.next_u32() % self.colors as u32) as u8);
                    }
                }
            }
            for (i, &c) in cells.iter().enumerate() {
                self.board.set(c, Some(pieces[i]));
            }
            if find_groups(&self.board).is_empty() && !moves(&self.board).is_empty() {
                return;
            }
        }
        // Still nothing: clear the rocks and deal a fresh board.
        self.board = Board { g: [[None; COLS]; ROWS] };
        self.fill();
    }

    /// The meter as the faces show it: 0 wide awake … 1 asleep.
    pub fn drowsiness(&self) -> f32 {
        ((DROWSY_FROM - self.energy) / DROWSY_FROM).clamp(0.0, 1.0)
    }
}
