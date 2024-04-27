(module
    (memory 1)

    ;; scalar mul vec4 (f64)
    (func $_rpu_scalar_mul_vec4_f64
        (param $scalar f64)  ;; Scalar
        (param $vec4_x f64)  ;; x component of vec4
        (param $vec4_y f64)  ;; y component of vec4
        (param $vec4_z f64)  ;; z component of vec4
        (param $vec4_w f64)  ;; w component of vec4
        (result f64 f64 f64 f64)  ;; Return four f64 results, the new x, y, z and w components

        ;; Calculate the new x component and return it
        (f64.mul
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec4_x)  ;; Get the x component
        )

        ;; Calculate the new y component and return it
        (f64.mul
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec4_y)  ;; Get the y component
        )

        ;; Calculate the new z component and return it
        (f64.mul
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec4_z)  ;; Get the z component
        )

        ;; Calculate the new w component and return it
        (f64.mul
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec4_w)  ;; Get the w component
        )
    )

    ;; vec4 div scalar (f64)
    (func $_rpu_vec4_div_scalar_f64
        (param $vec4_x f64)    ;; x component of vec4
        (param $vec4_y f64)    ;; y component of vec4
        (param $vec4_z f64)    ;; z component of vec4
        (param $vec4_w f64)    ;; w component of vec4
        (param $scalar f64)    ;; Scalar
        (result f64 f64 f64 f64)       ;; Return four f64 results, the new x, y, z and w components

        ;; Calculate the new x component and return it
        (f64.div
            (local.get $vec4_x)  ;; Get the x component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new y component and return it
        (f64.div
            (local.get $vec4_y)  ;; Get the y component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new z component and return it
        (f64.div
            (local.get $vec4_z)  ;; Get the z component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new w component and return it
        (f64.div
            (local.get $vec4_w)  ;; Get the w component
            (local.get $scalar)  ;; Get the scalar
        )
    )

    ;; function 'main'
    (func $main (export "main")  (result f64 f64 f64 f64)
        (local $result_x f64)
        (local $result_y f64)
        (local $result_z f64)
        (local $result_w f64)
        (f64.const 2)
        (f64.const 1)
        (f64.const 3)
        (f64.const 5)
        (f64.const 7)
        (call $_rpu_scalar_mul_vec4_f64)
        (f64.const 1.5)
        (call $_rpu_vec4_div_scalar_f64)
        local.set $result_w
        local.set $result_z
        local.set $result_y
        local.set $result_x
        local.get $result_x
        local.get $result_y
        local.get $result_z
        local.get $result_w
        
        (return)
    )
)
