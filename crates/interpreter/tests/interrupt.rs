#![allow(unused_crate_dependencies)]
use core::time;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;

use portable_atomic::AtomicBool;
use wasefire_interpreter::*;

#[test]
fn test_interrupt() {
    let mut n_interrupts = 0;
    let mut n_loops = 0;
    let interrupt = AtomicBool::new(false);

    std::thread::scope(|s: &std::thread::Scope<'_, '_>| {
        // Create an empty store.
        let mut store = Store::default();

        store.link_func("env", "count", 0, 1).unwrap();

        // ;; Use `wat2wasm infinite_loop.wat` to regenerate `.wasm`.
        // (module
        //     (import "env" "count" (func $count (result i32)))

        //     (memory (export "memory") 1)
        //         (func (export "loopforever")
        //             (local i32 i32)
        //             (loop
        //                 (local.set 0 (call $count))
        //                 (local.set 1 (i32.const 1))
        //                 (block
        //                     (loop
        //                         (br_if 1 (i32.gt_u (local.get 1) (local.get 0)))
        //                         (local.set 1 (i32.add (local.get 1) (i32.const 1)))
        //                         (br 0)
        //                     )
        //                 )
        //                 (br 0)
        //             )
        //         )
        // )

        const WASM: &[u8] = &[
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x00, 0x01,
            0x7f, 0x60, 0x00, 0x00, 0x02, 0x0d, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x05, 0x63, 0x6f,
            0x75, 0x6e, 0x74, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01,
            0x07, 0x18, 0x02, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x0b, 0x6c,
            0x6f, 0x6f, 0x70, 0x66, 0x6f, 0x72, 0x65, 0x76, 0x65, 0x72, 0x00, 0x01, 0x0a, 0x29,
            0x01, 0x27, 0x01, 0x02, 0x7f, 0x03, 0x40, 0x10, 0x00, 0x21, 0x00, 0x41, 0x01, 0x21,
            0x01, 0x02, 0x40, 0x03, 0x40, 0x20, 0x01, 0x20, 0x00, 0x4b, 0x0d, 0x01, 0x20, 0x01,
            0x41, 0x01, 0x6a, 0x21, 0x01, 0x0c, 0x00, 0x0b, 0x0b, 0x0c, 0x00, 0x0b, 0x0b,
        ];
        let module = Module::new(WASM).unwrap();
        let mut memory = [0; 16];

        // Instantiate the module in the store.
        let inst = store.instantiate(module, &mut memory).unwrap();

        store.set_interrupt(Some(&interrupt));
        let mut result = store.invoke(inst, "loopforever", vec![]).unwrap();

        // Let the outer infinite loop do 10 iterations.
        while n_loops <= 10 {
            let call = match result {
                RunResult::Host(call) => call,
                RunResult::Interrupt(call) => call,
                RunResult::Done(_) => unreachable!(),
            };

            if call.is_interrupt() {
                n_interrupts += 1;
                result = call.resume(&[]).unwrap();
            } else {
                // This is the count() function called in the loop header.
                assert!(call.index() == 0);
                n_loops += 1;
                // Interrupt.
                s.spawn(|| {
                    thread::sleep(time::Duration::from_millis(1));
                    interrupt.store(true, Relaxed);
                });
                result = call.resume(&[Val::I32(1000)]).unwrap();
            }
        }
    });
    assert!(n_interrupts > 9);
}
