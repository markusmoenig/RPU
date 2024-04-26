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

    ;; function 'doubler'
    (func $doubler (param $y i64) (result i64)

        local.get $y
        
        (i64.const 2)
        (i64.mul)
        (return)
    )

    ;; function 'main'
    (func $main (export "main") (param $x i64) (result i64)
        (local $result_x i64)
        (local $result_y i64)

        (i64.const 2)
        (i64.const 5)
        (i64.const 2)
        
        (call $_rpu_scalar_mul_vec2_i64)
        local.set $result_y
        local.set $result_x
        
        local.get $result_x
        
        (call $doubler)
        (return)
    )
)
