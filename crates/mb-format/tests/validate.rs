//! Validator rules, exercised with small hand-written modules and bundles.

use std::collections::BTreeMap;

use mb_format::{pack, validate};

const MANIFEST: &str = r#"
abi = 0
id = "dev.test.game"
name = "Test"
version = "0.1.0"
creator = "@test"
orientation = "portrait"
logical_size = [360, 640]
inputs = ["touch"]
perf_tier = "lite"
stdlib = { mb2d = 1 }
"#;

const EXPORTS: &str = r#"
  (memory (export "memory") 1 2048)
  (func (export "mb_init"))
  (func (export "mb_update") (param f32))
  (func (export "mb_render"))
  (func (export "mb_alloc") (param i32) (result i32) i32.const 0)
"#;

fn module(body: &str) -> Vec<u8> {
    wat::parse_str(format!("(module {body})")).unwrap()
}

fn icon(w: u32, h: u32) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    b.extend(w.to_be_bytes());
    b.extend(h.to_be_bytes());
    b.extend([8, 6, 0, 0, 0]);
    b
}

fn bundle(manifest: &str, wasm: Vec<u8>, extra: &[(&str, &[u8])]) -> Vec<u8> {
    let mut files = BTreeMap::new();
    files.insert("manifest.toml".to_string(), manifest.as_bytes().to_vec());
    files.insert("game.wasm".to_string(), wasm);
    files.insert("icon.png".to_string(), icon(256, 256));
    files.insert("src/src/lib.rs".to_string(), b"// source".to_vec());
    for (p, d) in extra {
        files.insert(p.to_string(), d.to_vec());
    }
    pack(&files).unwrap()
}

fn errors(manifest: &str, wasm: Vec<u8>) -> Vec<String> {
    validate(&bundle(manifest, wasm, &[])).errors
}

fn assert_error(errs: &[String], needle: &str) {
    assert!(errs.iter().any(|e| e.contains(needle)), "expected an error containing {needle:?}, got {errs:#?}");
}

#[test]
fn minimal_game_is_valid() {
    let wasm = module(&format!(r#"(import "mb" "mb2d_clear" (func (param i32))) {EXPORTS}"#));
    let r = validate(&bundle(MANIFEST, wasm, &[]));
    assert!(r.ok(), "{:#?}", r.errors);
}

#[test]
fn pack_is_deterministic() {
    let wasm = module(EXPORTS);
    assert_eq!(bundle(MANIFEST, wasm.clone(), &[]), bundle(MANIFEST, wasm, &[]));
}

#[test]
fn rejects_wasi_imports() {
    let wasm = module(&format!(r#"(import "wasi_snapshot_preview1" "fd_write" (func (param i32 i32 i32 i32) (result i32))) {EXPORTS}"#));
    assert_error(&errors(MANIFEST, wasm), "only the \"mb\" namespace");
}

#[test]
fn rejects_unknown_host_functions() {
    let wasm = module(&format!(r#"(import "mb" "mb_open_socket" (func)) {EXPORTS}"#));
    assert_error(&errors(MANIFEST, wasm), "unknown host function mb.mb_open_socket");
}

#[test]
fn rejects_wrong_signatures() {
    let wasm = module(&format!(r#"(import "mb" "mb2d_clear" (func (param f32))) {EXPORTS}"#));
    assert_error(&errors(MANIFEST, wasm), "mb.mb2d_clear has signature (f32) -> ()");
}

#[test]
fn rejects_undeclared_stdlib_sensors_and_capabilities() {
    let no_stdlib = MANIFEST.replace("stdlib = { mb2d = 1 }", "");
    let wasm = module(&format!(r#"(import "mb" "mb2d_clear" (func (param i32))) {EXPORTS}"#));
    assert_error(&errors(&no_stdlib, wasm), "without declaring stdlib { mb2d = 1 }");

    let wasm = module(&format!(r#"(import "mb" "mb_tilt" (func (param i32) (result i32))) {EXPORTS}"#));
    assert_error(&errors(MANIFEST, wasm), "sensors = [\"tilt\"]");

    let wasm = module(&format!(r#"(import "mb" "mb_store_set" (func (param i32 i32 i32 i32) (result i32))) {EXPORTS}"#));
    assert_error(&errors(MANIFEST, wasm), "capabilities = [\"store\"]");
}

#[test]
fn needs_must_be_declared_input_sensors() {
    let wasm = || module(EXPORTS);
    let ok = format!("{MANIFEST}sensors = [\"tilt\", \"haptics\"]\nneeds = [\"tilt\"]\n");
    assert!(errors(&ok, wasm()).is_empty(), "{:?}", errors(&ok, wasm()));

    let undeclared = format!("{MANIFEST}needs = [\"tilt\"]\n");
    assert_error(&errors(&undeclared, wasm()), "also needs sensors = [\"tilt\"]");

    let haptics = format!("{MANIFEST}sensors = [\"haptics\"]\nneeds = [\"haptics\"]\n");
    assert_error(&errors(&haptics, wasm()), "haptics is output-only");
}

#[test]
fn enforces_memory_maximum_by_tier() {
    let unbounded = EXPORTS.replace("1 2048", "1");
    assert_error(&errors(MANIFEST, module(&unbounded)), "must declare a maximum");

    let too_big = EXPORTS.replace("1 2048", "1 4096");
    assert_error(&errors(MANIFEST, module(&too_big)), "perf_tier Lite allows 2048");
    let full = MANIFEST.replace("\"lite\"", "\"full\"");
    assert!(errors(&full, module(&too_big)).is_empty());
}

#[test]
fn requires_exports_with_exact_signatures() {
    let missing = EXPORTS.replace(r#"(func (export "mb_render"))"#, "");
    assert_error(&errors(MANIFEST, module(&missing)), "missing required export function mb_render");

    let wrong = EXPORTS.replace(r#"(func (export "mb_update") (param f32))"#, r#"(func (export "mb_update") (param f64))"#);
    assert_error(&errors(MANIFEST, module(&wrong)), "export mb_update has signature (f64) -> ()");
}

#[test]
fn rejects_disallowed_wasm_features() {
    let shared = EXPORTS.replace("(memory (export \"memory\") 1 2048)", "(memory (export \"memory\") 1 2048 shared)");
    assert_error(&errors(MANIFEST, module(&shared)), "disallowed feature");
}

#[test]
fn checks_bundle_layout() {
    let wasm = module(EXPORTS);
    let bad = |extra: &[(&str, &[u8])]| validate(&bundle(MANIFEST, wasm.clone(), extra)).errors;
    assert_error(&bad(&[("../escape.txt", b"x")]), "'..'");
    assert_error(&bad(&[("notes.txt", b"x")]), "unexpected top-level file");
    assert_error(&bad(&[("assets/run.sh", b"x")]), "asset type .sh is not allowed");
    assert_error(&bad(&[("assets/A.png", b"x"), ("assets/a.png", b"y")]), "differ only by case");
}

#[test]
fn checks_icon_and_manifest_fields() {
    let mut files = BTreeMap::new();
    files.insert("manifest.toml".to_string(), MANIFEST.replace("dev.test.game", "Not An Id").into_bytes());
    files.insert("game.wasm".to_string(), module(EXPORTS));
    files.insert("icon.png".to_string(), icon(128, 128));
    let errs = validate(&pack(&files).unwrap()).errors;
    assert_error(&errs, "icon.png is 128×128");
    assert_error(&errs, "must be reverse-DNS");
    assert_error(&errs, "missing src/");
}

#[test]
fn checks_score_boards() {
    let wasm = || module(EXPORTS);
    let with = |extra: &str| format!("{MANIFEST}{extra}");
    let ok = with("capabilities = [\"score\"]\n[[scores]]\nboard = 0\nlabel = \"Height\"\norder = \"higher\"\n");
    assert!(errors(&ok, wasm()).is_empty(), "{:#?}", errors(&ok, wasm()));
    let no_cap = with("[[scores]]\nboard = 0\nlabel = \"Height\"\norder = \"higher\"\n");
    assert_error(&errors(&no_cap, wasm()), "need capabilities = [\"score\"]");
    let no_board = with("capabilities = [\"score\"]\n");
    assert_error(&errors(&no_board, wasm()), "needs at least one [[scores]] board");
    let dup = with("capabilities = [\"score\"]\n[[scores]]\nboard = 1\nlabel = \"A\"\norder = \"lower\"\nformat = \"milliseconds\"\n[[scores]]\nboard = 1\nlabel = \"B\"\norder = \"higher\"\n");
    assert_error(&errors(&dup, wasm()), "board numbers must be unique");
}

#[test]
fn daily_seed_is_stable_and_per_game() {
    // Locked: changing the derivation would change every game's daily challenge.
    let a = mb_format::daily_seed("dev.maimbrain.stack", "2026-10-06");
    assert_eq!(a, mb_format::daily_seed("dev.maimbrain.stack", "2026-10-06"));
    assert_ne!(a, mb_format::daily_seed("dev.maimbrain.stack", "2026-10-07"));
    assert_ne!(a, mb_format::daily_seed("dev.maimbrain.hello", "2026-10-06"));
    assert_eq!(a, 0x5e362aeb5507a232);
}

#[test]
fn gates_mb3d_on_its_stdlib() {
    let wasm = || module(&format!(r#"(import "mb" "mb3d_node" (func (result i32))) (import "mb" "mb3d_render" (func)) {EXPORTS}"#));
    assert_error(&errors(MANIFEST, wasm()), "without declaring stdlib { mb3d = 1 }");
    let both = MANIFEST.replace("stdlib = { mb2d = 1 }", "stdlib = { mb2d = 1, mb3d = 1 }");
    assert!(errors(&both, wasm()).is_empty(), "{:#?}", errors(&both, wasm()));
    let wrong = module(&format!(r#"(import "mb" "mb3d_node_transform" (func (param i32))) {EXPORTS}"#));
    assert_error(&errors(&both, wrong), "mb.mb3d_node_transform has signature (i32) -> ()");
}

fn glb(json: &str, declared_extra: i64) -> Vec<u8> {
    let mut j = json.as_bytes().to_vec();
    while j.len() % 4 != 0 {
        j.push(b' ');
    }
    let total = 20 + j.len();
    let mut b = b"glTF".to_vec();
    b.extend(2u32.to_le_bytes());
    b.extend(((total as i64 + declared_extra) as u32).to_le_bytes());
    b.extend((j.len() as u32).to_le_bytes());
    b.extend(0x4E4F534Au32.to_le_bytes());
    b.extend(j);
    b
}

#[test]
fn checks_glb_assets() {
    let wasm = || module(EXPORTS);
    let ok = glb(r#"{"asset":{"version":"2.0"}}"#, 0);
    let r = validate(&bundle(MANIFEST, wasm(), &[("assets/ship.glb", &ok), ("assets/hull.jpg", &[0xff, 0xd8])]));
    assert!(r.ok(), "{:#?}", r.errors);
    let short = glb(r#"{"asset":{"version":"2.0"}}"#, 8);
    assert_error(&validate(&bundle(MANIFEST, wasm(), &[("assets/ship.glb", &short)])).errors, "declared length");
    assert_error(&validate(&bundle(MANIFEST, wasm(), &[("assets/ship.glb", b"not a gltf file at all")])).errors, "not a binary glTF");
}
