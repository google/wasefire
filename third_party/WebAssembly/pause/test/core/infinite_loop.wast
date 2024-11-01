(module
    (func (export "loopforever")
        (loop
        (br 0)
        )
    )

    (func $recurseforever 
        call $recurseforever
    )
    (export "recurseforever" (func $recurseforever))
)

(assert_return (invoke "recurseforever") (i64.const 1111))
(assert_return (invoke "loopforever") (i64.const 1111))

(assert_trap 
    (module
        (func $loopforever
            (loop
            (br 0)
            )
        )

        (start $loopforever)
    )
"interrupt")