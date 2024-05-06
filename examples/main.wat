(module
    (import "env" "_rpu_rand" (func $_rpu_rand (result f64)))

    (memory 1)

    ;; function 'main'
    (func $main (export "main")  (result f64 f64)
        (local $result f64)
        (call $_rpu_rand)
        local.set $result

        local.get $result
        (f64.const 0.5)
        (f64.lt)
        (if
            (then
                (f64.const 0)
                (f64.const 0)
                (local.set $result)
                (local.set $result)
            )
            (else
                (f64.const 1)
                (f64.const 1)
                (local.set $result)
                (local.set $result)
            )
        )
        (local.get $result)
        (local.get $result)
        (return)
    )
)
