//! Saved data and leaderboards (SPEC §5.8). Declare `capabilities = ["store"]`
//! and/or `["score"]` (with `[[scores]]` boards) in manifest.toml.

use crate::ffi;

/// The saved value for `key`, if any. Saves persist across sessions and
/// game updates; the quota is 256 KB per game.
pub fn get(key: &str) -> Option<Vec<u8>> {
    let n = unsafe { ffi::mb_store_get(key.as_ptr(), key.len() as u32, std::ptr::null_mut(), 0) };
    if n < 0 {
        return None;
    }
    let mut v = vec![0u8; n as usize];
    unsafe { ffi::mb_store_get(key.as_ptr(), key.len() as u32, v.as_mut_ptr(), n as u32) };
    Some(v)
}

/// Saves `value` (empty deletes). False if over quota or the key is invalid (1–64 bytes).
pub fn set(key: &str, value: &[u8]) -> bool {
    unsafe { ffi::mb_store_set(key.as_ptr(), key.len() as u32, value.as_ptr(), value.len() as u32) == 0 }
}

pub fn get_u64(key: &str) -> Option<u64> {
    get(key).and_then(|v| v.try_into().ok()).map(u64::from_le_bytes)
}

pub fn set_u64(key: &str, value: u64) -> bool {
    set(key, &value.to_le_bytes())
}

/// Submits a score to a board declared in `[[scores]]`.
pub fn submit_score(board: u32, value: i64) -> bool {
    unsafe { ffi::mb_score_submit(board, value) == 0 }
}

/// Opens the platform's leaderboard overlay for `board`.
pub fn show_scores(board: u32) {
    unsafe { ffi::mb_score_show(board) }
}
