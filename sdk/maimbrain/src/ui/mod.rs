//! A UI kit on top of mb2d: themes, anti-aliased shapes, styled text,
//! icons, buttons and meters, layout helpers for the safe area and the
//! feed's card overlays, and ready-made title, results and HUD screens.
//! See docs/UI.md for a tour and recipes.
//!
//! ```ignore
//! use maimbrain::ui::{self, Layout, Theme, TitleCard};
//! let layout = Layout::new();
//! let mut title = TitleCard::new("BUBBLE", Theme::candy()).tagline("don't let it pop").best(best);
//! // update: title.update(dt);   render: title.draw(&layout);
//! ```
//!
//! Draw calls follow mb2d's rules (transforms apply; nothing is retained),
//! and widget state (button presses, counters, entrances) advances only in
//! `update(dt)`, so the kit replays exactly (SPEC §3).

pub mod color;
pub mod icon;
pub mod layout;
pub mod screens;
pub mod shape;
pub mod text;
pub mod theme;
pub mod widgets;

pub use color::{darken, fade, hex, hsv, lighten, mix, rgba, with_alpha};
pub use icon::Icon;
pub use layout::{Anchor, Column, Layout, Rect, stack_h, stack_v};
pub use screens::{Countdown, Hud, ResultsAction, ResultsCard, ResultsEvent, TitleCard};
pub use text::{Align, VAlign, fmt_int, text};
pub use theme::Theme;
pub use widgets::{Button, ButtonKind, Tap, panel, pill, pill_width, progress_bar, ribbon, ring_meter, segments};
