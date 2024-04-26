(module
    (memory 1)

    ;; scalar mul vec2 (i64)
    (func $_rpu_scalar_mul_vec2_i64
        (param $scalar i64)  ;; Scalar
        (param $vec2_x i64)  ;; x component of vec2
        (param $vec2_y i64)  ;; y component of vec2
        (result i64 i64)  ;; Return two i64 results, the new x and y components

        ;; Calculate the new x component and return it
        (i64.mul
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec2_x)  ;; Get the x component
        )

        ;; Calculate the new y component and return it
        (i64.mul
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec2_y)  ;; Get the y component
        )
    )

    ;; function 'fib'
    (func $fib (param $n i64) (result i64)


        local.get $n
        
        (i64.const 1)
        (i64.le_s)

        (if
            (then
                local.get $n
                
                (return)
            )
        )

        
        local.get $n
        
        (i64.const 2)
        (i64.sub)
        (call $fib)
        
        local.get $n
        
        (i64.const 1)
        (i64.sub)
        (call $fib)
        (i64.add)
        (return)
    )

    ;; function 'main'
    (func $main (export "main") (param $x i64) (result i64)
        (local $result_x i64)
        (local $result_y i64)
        (local $test i64)

        (i64.const 2)
        (i64.const 5)
        (i64.const 2)
        
        (call $_rpu_scalar_mul_vec2_i64)
        local.set $result_y
        local.set $result_x
        (i64.const 0)
        local.set $test

        (i64.const 2)
        (i64.const 2)
        (i64.eq)

        (if
            (then
                (i64.const 3)
                local.set $test
            )
            (else
                (i64.const 4)
                local.set $test
            )
        )

        
        local.get $x
        
        (call $fib)
        local.set $test
        local.get $test
        
        (return)
    )
)
