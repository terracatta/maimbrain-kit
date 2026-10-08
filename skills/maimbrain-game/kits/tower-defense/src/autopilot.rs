//! A sensible tower-building heuristic, pure Rust: the title card's attract
//! mode plays with it, and the test bot (src/tests.rs) layers human
//! slowness and mistakes on top of it.

use crate::sim::{CreepKind, HEARTS, PADS, Sim, TowerKind, coverage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Build(usize, TowerKind),
    Upgrade(usize),
    CallWave,
}

impl Action {
    /// Coins it needs.
    pub fn cost(self, sim: &Sim) -> u32 {
        match self {
            Action::Build(_, k) => k.spec().cost[0],
            Action::Upgrade(p) => sim.upgrade_cost(p).unwrap_or(u32::MAX),
            Action::CallWave => 0,
        }
    }

    /// Does it to the sim; true if it happened.
    pub fn apply(self, sim: &mut Sim) -> bool {
        match self {
            Action::Build(p, k) => sim.build(p, k),
            Action::Upgrade(p) => sim.upgrade(p),
            Action::CallWave => sim.call_wave().is_some(),
        }
    }
}

/// How the planner plays.
#[derive(Clone, Copy, Debug)]
pub struct Style {
    /// Stop building past this many towers (attract mode caps it, so the
    /// cake falls now and then and the card shows the drama).
    pub max_towers: usize,
    /// Only build on pads at least this far down the screen (attract mode
    /// keeps the fight in view, below the title).
    pub pads_below: f32,
    pub upgrades: bool,
    pub call_early: bool,
}


pub struct Planner {
    /// Road covered by each pad at each tower kind's base range.
    cov: Vec<[f32; 3]>,
}

impl Planner {
    pub fn new(sim: &Sim) -> Planner {
        let cov = (0..PADS.len()).map(|p| TowerKind::ALL.map(|k| coverage(sim, p, k.spec().range[0]))).collect();
        Planner { cov }
    }

    pub fn coverage(&self, pad: usize, kind: TowerKind) -> f32 {
        self.cov[pad][kind as usize]
    }

    /// Empty pads, best first for `kind`.
    pub fn pads_for(&self, sim: &Sim, kind: TowerKind) -> Vec<usize> {
        let mut v: Vec<usize> = (0..PADS.len()).filter(|&p| sim.towers[p].is_none()).collect();
        v.sort_by(|&a, &b| self.coverage(b, kind).total_cmp(&self.coverage(a, kind)).then(a.cmp(&b)));
        v
    }

    /// The tower kind a sensible player adds next.
    pub fn next_kind(&self, sim: &Sim) -> TowerKind {
        let count = |k| sim.towers.iter().flatten().filter(|t| t.kind == k).count();
        let (pops, chills, booms) = (count(TowerKind::Pop), count(TowerKind::Chill), count(TowerKind::Boom));
        let total = pops + chills + booms;
        let tough_ahead = sim.wave >= 4 || sim.next.groups.iter().any(|g| matches!(g.0, CreepKind::Helmet | CreepKind::Splitter | CreepKind::King));
        if total == 0 {
            TowerKind::Pop
        } else if tough_ahead && booms == 0 {
            TowerKind::Boom
        } else if total >= 3 && chills == 0 {
            TowerKind::Chill
        } else if tough_ahead && booms * 2 < pops {
            TowerKind::Boom
        } else {
            TowerKind::Pop
        }
    }

    /// Rough damage per second a tower adds, for comparing options.
    fn dps(kind: TowerKind, level: usize) -> f32 {
        let s = kind.spec();
        let base = s.damage[level] / s.interval[level];
        match kind {
            TowerKind::Pop => base,
            TowerKind::Chill => base * 2.5 + (1.0 - s.special[level]) * 2.0,
            TowerKind::Boom => base * 2.0,
        }
    }

    /// What to do next (which may need saving up for), or None.
    pub fn want(&self, sim: &Sim, style: &Style) -> Option<Action> {
        if !sim.playing() {
            return None;
        }
        let towers = sim.towers.iter().flatten().count();
        let kind = self.next_kind(sim);
        let build = (towers < style.max_towers).then(|| self.pads_for(sim, kind).into_iter().find(|&p| PADS[p].1 >= style.pads_below)).flatten();
        // Value per coin of building vs the best upgrade.
        let build_value = build.map(|p| self.coverage(p, kind) * Self::dps(kind, 0) / kind.spec().cost[0] as f32);
        let mut upgrade: Option<(usize, f32)> = None;
        if style.upgrades && towers >= 3 {
            for p in 0..PADS.len() {
                let (Some(t), Some(cost)) = (&sim.towers[p], sim.upgrade_cost(p)) else { continue };
                let gain = Self::dps(t.kind, t.level + 1) - Self::dps(t.kind, t.level);
                let v = self.coverage(p, t.kind) * gain / cost as f32;
                if upgrade.is_none_or(|u| v > u.1) {
                    upgrade = Some((p, v));
                }
            }
        }
        let action = match (build, build_value, upgrade) {
            (Some(p), Some(bv), Some((u, uv))) => {
                if towers < 4 || bv * 1.1 >= uv { Some(Action::Build(p, kind)) } else { Some(Action::Upgrade(u)) }
            }
            (Some(p), _, None) => Some(Action::Build(p, kind)),
            (None, _, Some((u, _))) => Some(Action::Upgrade(u)),
            _ => None,
        };
        // Calling early: only with the board calm, full hearts and money spent.
        if style.call_early && sim.wave >= 2 && sim.pending_spawns() == 0 && sim.hearts == HEARTS && sim.next_in > 3.0 {
            let far = sim.creeps.iter().all(|c| c.d < sim.road_len() * 0.3);
            let broke = action.is_none_or(|a| a.cost(sim) > sim.coins + 20);
            if far && broke && sim.creeps.len() < 4 {
                return Some(Action::CallWave);
            }
        }
        action
    }
}
