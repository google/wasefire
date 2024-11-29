#![allow(unused_crate_dependencies)]
use core::time;
use std::sync::atomic::Ordering::SeqCst;
use std::thread;

use portable_atomic::{AtomicBool, AtomicI32};
use wasefire_interpreter::*;

#[test]
fn test_interrupt() {
    let n_interrupts = AtomicI32::new(0);
    let n_loops = AtomicI32::new(0);
    let interrupt = AtomicBool::new(false);

    std::thread::scope(|s: &std::thread::Scope<'_, '_>| {
        s.spawn(|| {
            // Create an empty store.
            let mut store = Store::default();

            store.link_func("env", "count", 0, 1).unwrap();

            const WASM: &[u8] = include_bytes!("infinite_loop.wasm");
            let module = Module::new(WASM).unwrap();
            let mut memory = [0; 16];

            // Instantiate the module in the store.
            let inst = store.instantiate(module, &mut memory).unwrap();

            store.set_interrupt(Some(&interrupt));
            let mut result = store.invoke(inst, "loopforever", vec![]).unwrap();

            // Let the outer infinite loop do 10 iterations.
            while n_loops.load(SeqCst) <= 10 {
                let call = match result {
                    RunResult::Host(call) => call,
                    RunResult::Interrupt(call) => call,
                    RunResult::Done(_) => unreachable!(),
                };

                if n_loops.load(SeqCst) > 10 {
                    break;
                }

                if call.is_interrupt() {
                    n_interrupts.add(1, SeqCst);
                    result = call.resume(&[]).unwrap();
                } else {
                    // This is the count() function called in the loop header.
                    assert!(call.index() == 0);
                    n_loops.add(1, SeqCst);
                    // Interrupt.
                    s.spawn(|| {
                        thread::sleep(time::Duration::from_millis(1));
                        interrupt.store(true, SeqCst);
                    });
                    result = call.resume(&[Val::I32(1000)]).unwrap();
                }
            }
        });

        thread::sleep(time::Duration::from_millis(100));
        assert!(n_loops.load(SeqCst) > 9 && n_interrupts.load(SeqCst) > 9);
    });
}
