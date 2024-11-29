;; Use `wat2wasm infinite_loop.wat` to regenerate `.wasm`.
(module
    (import "env" "count" (func $count (result i32)))   

    (memory (export "memory") 1)
        (func (export "loopforever")
            (local i32 i32)
            (loop
                (local.set 0 (call $count))
                (local.set 1 (i32.const 1))
                (block
                    (loop
                        (br_if 1 (i32.gt_u (local.get 1) (local.get 0)))
                        (local.set 1 (i32.add (local.get 1) (i32.const 1)))
                        (br 0)
                    )
                )
                (br 0)
            )
        )
)