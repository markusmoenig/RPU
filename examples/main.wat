(module

    (memory 1)

    ;; function 'main'
    (func $main (export "main")  (result f64)
        (local $result f64)
        (local $a f64)
        (local $b f64)
        (f64.const 0)
        local.set $result
        (f64.const 2)
        local.set $a
        (f64.const 4)
        local.set $b

        local.get $b
        (f64.const 4)
        (f64.eq)
        local.get $a
        (f64.const 3)
        (f64.eq)
        (i32.and)
        (if
            (then
                (f64.const 1)
                local.set $result
            )
            (else
                (f64.const 2)
                local.set $result
            )
        )
        local.get $result
        (return)
    )
)
