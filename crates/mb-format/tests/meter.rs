//! Loop metering: spinning loops trap, bounded loops run and consume fuel.

use mb_format::meter::{FUEL_EXPORT, instrument};
use wasmi::{Engine, Instance, Linker, Module, Store, Val};

fn metered(wat: &str) -> (Store<()>, Instance) {
    let wasm = instrument(&wat::parse_str(wat).unwrap()).unwrap();
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm).unwrap();
    let mut store = Store::new(&engine, ());
    let instance = Linker::new(&engine).instantiate_and_start(&mut store, &module).unwrap();
    (store, instance)
}

fn set_fuel(store: &mut Store<()>, i: &Instance, v: i32) {
    i.get_global(&*store, FUEL_EXPORT).unwrap().set(store, Val::I32(v)).unwrap();
}

fn fuel(store: &Store<()>, i: &Instance) -> i32 {
    i.get_global(store, FUEL_EXPORT).unwrap().get(store).i32().unwrap()
}

#[test]
fn infinite_loop_traps_when_fuel_runs_out() {
    let (mut store, i) = metered(r#"(module (func (export "spin") (loop br 0)))"#);
    set_fuel(&mut store, &i, 1_000_000);
    let spin = i.get_typed_func::<(), ()>(&store, "spin").unwrap();
    assert!(spin.call(&mut store, ()).is_err());
    assert_eq!(fuel(&store, &i), 0);
}

#[test]
fn bounded_loops_run_and_consume_one_fuel_per_iteration() {
    let (mut store, i) = metered(
        r#"(module (func (export "count") (param $n i32) (result i32) (local $i i32)
             (loop $l
               (local.set $i (i32.add (local.get $i) (i32.const 1)))
               (br_if $l (i32.lt_u (local.get $i) (local.get $n))))
             (local.get $i)))"#,
    );
    set_fuel(&mut store, &i, 100);
    let count = i.get_typed_func::<i32, i32>(&store, "count").unwrap();
    assert_eq!(count.call(&mut store, 10).unwrap(), 10);
    assert_eq!(fuel(&store, &i), 90);
}

#[test]
fn nested_loops_are_all_metered() {
    let (mut store, i) = metered(r#"(module (func (export "spin") (loop (loop br 0))))"#);
    set_fuel(&mut store, &i, 50);
    let spin = i.get_typed_func::<(), ()>(&store, "spin").unwrap();
    assert!(spin.call(&mut store, ()).is_err());
}
