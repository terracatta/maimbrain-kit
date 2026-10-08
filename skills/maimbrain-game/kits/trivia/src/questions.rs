//! The question bank: `src/questions.txt`, one question per line
//! (`tier | CATEGORY | question | right | wrong | wrong | wrong`), compiled
//! into the game with `include_str!`.
//!
//! Why a text file compiled in, not an asset loaded at run time: the round
//! needs its questions the moment the first tap lands, and assets load
//! asynchronously; compiling it in means no loading state, no fallback and
//! nothing that can differ between a run and its replay. It stays the most
//! reskinnable form there is (edit the text, rebuild: `mb serve --watch`
//! rebuilds on save), and `cargo test -p kit_trivia` checks every line.

use std::sync::OnceLock;

/// Limits the card and the answer buttons are laid out for.
pub const MAX_QUESTION_CHARS: usize = 96;
pub const MAX_ANSWER_CHARS: usize = 22;

/// The raw bank, as written.
pub const RAW: &str = include_str!("questions.txt");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Question {
    /// 1 easy, 2 medium, 3 hard.
    pub tier: u8,
    /// Any word; `look::category` gives known ones an icon and a color.
    pub cat: &'static str,
    pub text: &'static str,
    /// `answers[0]` is the right one; the sim shuffles them for the screen.
    pub answers: [&'static str; 4],
}

/// Parses a bank; errors name the line.
pub fn parse(src: &'static str) -> Result<Vec<Question>, String> {
    let mut out = Vec::new();
    for (n, line) in src.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&'static str> = line.split('|').map(str::trim).collect();
        if f.len() != 7 {
            return Err(format!("line {}: expected 7 fields separated by |, found {}", n + 1, f.len()));
        }
        let tier = match f[0] {
            "1" => 1,
            "2" => 2,
            "3" => 3,
            t => return Err(format!("line {}: tier must be 1, 2 or 3, not {t:?}", n + 1)),
        };
        if f.iter().any(|s| s.is_empty()) {
            return Err(format!("line {}: an empty field", n + 1));
        }
        if f[2].chars().count() > MAX_QUESTION_CHARS {
            return Err(format!("line {}: the question is longer than {MAX_QUESTION_CHARS} characters", n + 1));
        }
        if let Some(a) = f[3..].iter().find(|a| a.chars().count() > MAX_ANSWER_CHARS) {
            return Err(format!("line {}: the answer {a:?} is longer than {MAX_ANSWER_CHARS} characters", n + 1));
        }
        out.push(Question { tier, cat: f[1], text: f[2], answers: [f[3], f[4], f[5], f[6]] });
    }
    Ok(out)
}

/// The game's bank, parsed once. A malformed line panics here (and fails
/// `cargo test` first).
pub fn bank() -> &'static [Question] {
    static BANK: OnceLock<Vec<Question>> = OnceLock::new();
    BANK.get_or_init(|| parse(RAW).expect("src/questions.txt"))
}
