//! Loop metering: guarantees a guest callback can't run forever.
//!
//! On iOS neither releasing a WKWebView nor `Worker.terminate()` stops a wasm
//! loop that never yields (Phase 0/1 measurements), so the client rewrites
//! `game.wasm` before serving it: every `loop` starts by decrementing a fuel
//! counter and traps when it hits zero. The runtime refills the counter
//! (exported as `__mb_fuel`) before each callback. Without loops, wasm can't
//! run unboundedly: recursion hits the stack limit and bulk memory operations
//! are bounded by memory size. Exhaustion is deterministic, so it replays.

use walrus::ir::{BinaryOp, Instr, InstrLocId, InstrSeqId, UnaryOp, Value, Visitor, dfs_in_order};
use walrus::{ConstExpr, GlobalId, Module, ValType};

/// Name of the exported fuel global.
pub const FUEL_EXPORT: &str = "__mb_fuel";
/// Exports with this prefix are reserved for the platform.
pub const RESERVED_PREFIX: &str = "__mb";

/// Returns the metered module.
pub fn instrument(wasm: &[u8]) -> Result<Vec<u8>, String> {
    let mut module = Module::from_buffer(wasm).map_err(|e| format!("metering: {e}"))?;
    let fuel = module.globals.add_local(ValType::I32, true, false, ConstExpr::Value(Value::I32(i32::MAX)));
    module.exports.add(FUEL_EXPORT, fuel);

    for (_, func) in module.funcs.iter_local_mut() {
        let mut loops = Loops::default();
        dfs_in_order(&mut loops, func, func.entry_block());
        let builder = func.builder_mut();
        for seq in loops.0 {
            let trap = builder.dangling_instr_seq(None).unreachable().id();
            let ok = builder.dangling_instr_seq(None).id();
            check(&mut builder.instr_seq(seq), fuel, trap, ok);
        }
    }
    Ok(module.emit_wasm())
}

/// `if (fuel == 0) unreachable; fuel -= 1;` at the start of a loop body.
fn check(seq: &mut walrus::InstrSeqBuilder<'_>, fuel: GlobalId, trap: InstrSeqId, ok: InstrSeqId) {
    let prologue: [Instr; 7] = [
        walrus::ir::GlobalGet { global: fuel }.into(),
        walrus::ir::Unop { op: UnaryOp::I32Eqz }.into(),
        walrus::ir::IfElse { consequent: trap, alternative: ok }.into(),
        walrus::ir::GlobalGet { global: fuel }.into(),
        walrus::ir::Const { value: Value::I32(1) }.into(),
        walrus::ir::Binop { op: BinaryOp::I32Sub }.into(),
        walrus::ir::GlobalSet { global: fuel }.into(),
    ];
    for (i, instr) in prologue.into_iter().enumerate() {
        seq.instr_at(i, instr);
    }
}

#[derive(Default)]
struct Loops(Vec<InstrSeqId>);

impl<'i> Visitor<'i> for Loops {
    fn visit_instr(&mut self, instr: &'i Instr, _: &'i InstrLocId) {
        if let Instr::Loop(l) = instr {
            self.0.push(l.seq);
        }
    }
}
