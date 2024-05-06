(module
    (import "env" "_rpu_rand" (func $_rpu_rand (result f64)))

    (memory 1)

    ;; function 'opU'
    (func $opU (param $o1_x f64) (param $o1_y f64)(param $o2_x f64) (param $o2_y f64) (result f64 f64)
        (local $_rpu_ternary_0_x f64)
        (local $_rpu_ternary_0_y f64)

        local.get $o1_x
        local.get $o2_x
        (f64.lt)
        (if
            (then
                local.get $o1_x
                local.get $o1_y
                (local.set $_rpu_ternary_0_y)
                (local.set $_rpu_ternary_0_x)
            )
            (else
                local.get $o2_x
                local.get $o2_y
                (local.set $_rpu_ternary_0_y)
                (local.set $_rpu_ternary_0_x)
            )
        )
        (local.get $_rpu_ternary_0_x)
        (local.get $_rpu_ternary_0_y)
        (return)
    )

    ;; function 'main'
    (func $main (export "main")  (result f64 f64)
        (local $result f64)
        (local $_rpu_ternary_1_x f64)
        (local $_rpu_ternary_1_y f64)
        (call $_rpu_rand)
        local.set $result

        local.get $result
        (f64.const 0.5)
        (f64.lt)
        (if
            (then
                (f64.const 0)
                (f64.const 0.1)
                (local.set $_rpu_ternary_1_y)
                (local.set $_rpu_ternary_1_x)
            )
            (else
                (f64.const 1)
                (f64.const 1.1)
                (local.set $_rpu_ternary_1_y)
                (local.set $_rpu_ternary_1_x)
            )
        )
        (local.get $_rpu_ternary_1_x)
        (local.get $_rpu_ternary_1_y)
        (return)
    )
)
