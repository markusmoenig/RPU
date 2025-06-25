(module

    (memory 1)


    ;; function 'fib'
    (func $fib (param $n_0 i64) (result i64)

        local.get $n_0
        (i64.const 1)
        (i64.le_s)
        (if
            (then
                local.get $n_0
                (return)
            )
        )
        local.get $n_0
        (i64.const 2)
        (i64.sub)
        (call $fib)
        local.get $n_0
        (i64.const 1)
        (i64.sub)
        (call $fib)
        (i64.add)
        (return)
    )

    ;; function 'main'
    (func $main (export "main") (param $x_1 i64) (result i64)
        local.get $x_1
        (call $fib)
        (return)
    )
)
