//! Playing a glTF model's animation clips (mb3d 2): [`Clip`] names one,
//! [`Node::pose`] adds one weighted sample of it to a spawned model's pose
//! this frame, and [`Animator`] does the bookkeeping most games want: play,
//! loop or play once, speed, and crossfades between clips (idle → walk →
//! run), with walk and run kept in step.
//!
//! Animation time is yours: the animator only moves when you call
//! [`Animator::update`] with your frame's `dt`, and the host samples clips at
//! exactly the times you pass, so replays pose the model identically.

use super::{Model, Node};
use crate::ffi;

/// One of a model's animation clips, with its length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clip {
    pub index: u32,
    /// Seconds.
    pub duration: f32,
}

impl Model {
    /// How many animation clips the model has.
    pub fn clip_count(self) -> u32 {
        unsafe { ffi::mb3d_clip_count(self.0.get()) }.max(0) as u32
    }
    /// The clip named `name` (its glTF animation name), if the model has one.
    pub fn clip(self, name: &str) -> Option<Clip> {
        let i = unsafe { ffi::mb3d_clip_find(self.0.get(), name.as_ptr(), name.len() as u32) };
        (i >= 0).then(|| self.clip_at(i as u32)).flatten()
    }
    /// Clip number `index` (0 ≤ index < [`clip_count`](Self::clip_count)).
    pub fn clip_at(self, index: u32) -> Option<Clip> {
        let duration = unsafe { ffi::mb3d_clip_duration(self.0.get(), index) };
        (duration >= 0.0).then_some(Clip { index, duration })
    }
}

impl Node {
    /// Adds `clip` sampled at `time` seconds (clamped to the clip) with
    /// `weight` to this frame's pose of the model spawned at this node (the
    /// root [`Model::spawn`] returned). Samples blend by weight; weights short
    /// of 1 are made up by the model's rest pose. Call it every frame you want
    /// the pose to change (in `update` or `render`, before `render()`); with
    /// no samples a frame keeps the last pose. False if this isn't a spawned
    /// model's root or the clip doesn't exist.
    pub fn pose(self, clip: Clip, time: f32, weight: f32) -> bool {
        unsafe { ffi::mb3d_anim(self.0.get(), clip.index, time, weight) >= 0 }
    }
}

/// How a clip plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Play {
    /// Repeats forever.
    Loop,
    /// Plays to the end and holds the last frame ([`Animator::finished`] turns true).
    Once,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Layer {
    clip: Clip,
    mode: Play,
    time: f32,
    speed: f32,
    /// 0–1 and how fast it moves toward `target` (per second).
    weight: f32,
    target: f32,
    rate: f32,
}

/// Plays clips on one spawned model with crossfades. Keep one per model
/// instance in your game state, `update` it with `dt` and `apply` it each frame.
///
/// ```ignore
/// let idle = model.clip("Idle").unwrap();
/// let run = model.clip("Run").unwrap();
/// let mut anim = Animator::new();
/// anim.play(idle, Play::Loop, 0.0);
/// // later, when the player starts running:
/// anim.play_synced(run, 0.25);
/// // every frame:
/// anim.update(dt);
/// anim.apply(fox);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Animator {
    layers: Vec<Layer>,
}

/// Layers kept at once: the current clip and the ones fading out.
const MAX_LAYERS: usize = 4;

impl Animator {
    pub fn new() -> Animator {
        Animator::default()
    }

    /// Crossfades to `clip` over `fade` seconds (0 = cut), from its start.
    /// Playing the clip that's already current changes nothing, except that
    /// a finished [`Play::Once`] clip starts over.
    pub fn play(&mut self, clip: Clip, mode: Play, fade: f32) {
        if let Some(cur) = self.layers.last()
            && cur.clip == clip
            && cur.target == 1.0
            && !(cur.mode == Play::Once && self.finished())
        {
            self.layers.last_mut().unwrap().mode = mode;
            return;
        }
        self.start(clip, mode, fade, 0.0);
    }

    /// Like [`play`](Self::play) with [`Play::Loop`], but starting at the same
    /// point of its cycle as the current clip (its time as a fraction of its
    /// length), so a walk and a run cycle stay in step: feet don't skip when
    /// one fades into the other.
    pub fn play_synced(&mut self, clip: Clip, fade: f32) {
        if self.layers.last().is_some_and(|c| c.clip == clip && c.target == 1.0) {
            return;
        }
        let phase = self.phase();
        self.start(clip, Play::Loop, fade, phase * clip.duration);
    }

    /// Plays `clip` from the start even if it's current (a second jump).
    pub fn restart(&mut self, clip: Clip, mode: Play, fade: f32) {
        self.start(clip, mode, fade, 0.0);
    }

    fn start(&mut self, clip: Clip, mode: Play, fade: f32, time: f32) {
        let rate = if fade > 1e-4 { 1.0 / fade } else { f32::INFINITY };
        for l in &mut self.layers {
            l.target = 0.0;
            l.rate = rate;
        }
        let first = self.layers.is_empty();
        let speed = self.layers.last().map_or(1.0, |l| if l.clip == clip { l.speed } else { 1.0 });
        self.layers.push(Layer { clip, mode, time, speed, weight: if first || rate.is_infinite() { 1.0 } else { 0.0 }, target: 1.0, rate });
        if rate.is_infinite() {
            self.layers.retain(|l| l.target > 0.0);
        }
        while self.layers.len() > MAX_LAYERS {
            self.layers.remove(0);
        }
    }

    /// Playback speed of the current clip (1 = as authored; e.g. match a run to the ground speed).
    pub fn set_speed(&mut self, speed: f32) {
        if let Some(l) = self.layers.last_mut() {
            l.speed = speed.max(0.0);
        }
    }

    /// Advances every playing clip and crossfade by `dt` seconds (your frame's dt).
    pub fn update(&mut self, dt: f32) {
        if !(dt > 0.0) {
            return;
        }
        for l in &mut self.layers {
            l.time += dt * l.speed;
            if l.clip.duration > 0.0 {
                match l.mode {
                    Play::Loop => l.time = l.time.rem_euclid(l.clip.duration),
                    Play::Once => l.time = l.time.min(l.clip.duration),
                }
            }
            let step = if l.rate.is_infinite() { 1.0 } else { l.rate * dt };
            l.weight = if l.weight < l.target { (l.weight + step).min(l.target) } else { (l.weight - step).max(l.target) };
        }
        self.layers.retain(|l| l.target > 0.0 || l.weight > 0.0);
    }

    /// Poses `node` (a spawned model's root) with every audible layer.
    pub fn apply(&self, node: Node) {
        for (clip, time, w) in self.samples() {
            node.pose(clip, time, w);
        }
    }

    /// What [`apply`](Self::apply) sends: (clip, time, weight) per layer, crossfades eased.
    pub fn samples(&self) -> impl Iterator<Item = (Clip, f32, f32)> + '_ {
        self.layers.iter().filter(|l| l.weight > 0.0).map(|l| {
            let w = l.weight;
            (l.clip, l.time, w * w * (3.0 - 2.0 * w))
        })
    }

    /// The clip playing (or fading in) now.
    pub fn current(&self) -> Option<Clip> {
        self.layers.last().map(|l| l.clip)
    }

    /// Seconds into the current clip.
    pub fn time(&self) -> f32 {
        self.layers.last().map_or(0.0, |l| l.time)
    }

    /// How far through its cycle the current clip is (0–1).
    pub fn phase(&self) -> f32 {
        self.layers.last().map_or(0.0, |l| if l.clip.duration > 0.0 { l.time / l.clip.duration } else { 0.0 })
    }

    /// The current clip is [`Play::Once`] and has reached its end.
    pub fn finished(&self) -> bool {
        self.layers.last().is_some_and(|l| l.mode == Play::Once && l.time >= l.clip.duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDLE: Clip = Clip { index: 0, duration: 2.0 };
    const WALK: Clip = Clip { index: 1, duration: 1.0 };
    const RUN: Clip = Clip { index: 2, duration: 0.5 };
    const JUMP: Clip = Clip { index: 3, duration: 0.8 };

    fn run(a: &mut Animator, seconds: f32) {
        for _ in 0..(seconds * 60.0).round() as u32 {
            a.update(1.0 / 60.0);
        }
    }

    #[test]
    fn loops_and_crossfades_with_weights_summing_to_one() {
        let mut a = Animator::new();
        a.play(IDLE, Play::Loop, 0.3);
        assert_eq!(a.samples().collect::<Vec<_>>(), vec![(IDLE, 0.0, 1.0)], "the first clip starts at full weight");
        run(&mut a, 2.5);
        assert!((a.time() - 0.5).abs() < 1e-3, "looped: {}", a.time());
        a.play(WALK, Play::Loop, 0.5);
        run(&mut a, 0.25);
        let s: Vec<_> = a.samples().collect();
        assert_eq!(s.len(), 2);
        let total: f32 = s.iter().map(|x| x.2).sum();
        assert!((total - 1.0).abs() < 1e-4, "{s:?}");
        run(&mut a, 0.3);
        assert_eq!(a.samples().map(|x| x.0).collect::<Vec<_>>(), vec![WALK], "the old clip is dropped once faded out");
        // Playing the current clip again changes nothing.
        let before = a.clone();
        a.play(WALK, Play::Loop, 0.2);
        assert_eq!(a, before);
    }

    #[test]
    fn synced_crossfades_keep_the_cycle_phase() {
        let mut a = Animator::new();
        a.play(WALK, Play::Loop, 0.0);
        run(&mut a, 0.25);
        a.play_synced(RUN, 0.2);
        assert!((a.phase() - 0.25).abs() < 1e-3 && (a.time() - 0.125).abs() < 1e-3, "{} {}", a.phase(), a.time());
    }

    #[test]
    fn once_holds_its_last_frame_and_can_restart() {
        let mut a = Animator::new();
        a.play(IDLE, Play::Loop, 0.0);
        a.play(JUMP, Play::Once, 0.1);
        run(&mut a, 0.5);
        assert!(!a.finished());
        run(&mut a, 0.5);
        assert!(a.finished() && a.time() == JUMP.duration);
        a.play(JUMP, Play::Once, 0.1);
        assert_eq!(a.time(), 0.0, "a finished one-shot starts over");
        a.set_speed(2.0);
        run(&mut a, 0.2);
        assert!((a.time() - 0.4).abs() < 1e-3);
    }

    #[test]
    fn cuts_and_layer_cap() {
        let mut a = Animator::new();
        for c in [IDLE, WALK, RUN, JUMP, IDLE, WALK] {
            a.restart(c, Play::Loop, 0.5);
        }
        assert!(a.layers.len() <= MAX_LAYERS);
        a.play(RUN, Play::Loop, 0.0);
        assert_eq!(a.samples().collect::<Vec<_>>(), vec![(RUN, 0.0, 1.0)], "a cut drops everything else");
    }
}
