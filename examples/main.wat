(module
    (memory 1)

    ;; vec2 sub scalar (f64)
    (func $_rpu_vec2_sub_scalar_f64
        (param $vec2_x f64)    ;; x component of vec2
        (param $vec2_y f64)    ;; y component of vec2
        (param $scalar f64)    ;; Scalar
        (result f64 f64)       ;; Return two f64 results, the new x and y components

        ;; Calculate the new x component and return it
        (f64.sub
            (local.get $vec2_x)  ;; Get the x component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new y component and return it
        (f64.sub
            (local.get $vec2_y)  ;; Get the y component
            (local.get $scalar)  ;; Get the scalar
        )
    )

    ;; vec2 length
    (func $_rpu_vec2_length_f64 (param $x f64) (param $y f64) (result f64)        
        local.get $x
        local.get $x
        f64.mul
        local.get $y
        local.get $y
        f64.mul
        f64.add
        f64.sqrt)

    ;; vec1 smoothstep
    (func $_rpu_smoothstep_vec1_f64 (param $edge0_x f64) (param $edge1_x f64) (param $x f64) (result f64)
 (local $t_x f64)        
        ;; Calculate normalized t for the component x
        local.get $x
        local.get $edge0_x
        f64.sub
        local.get $edge1_x
        local.get $edge0_x
        f64.sub
        f64.div
        local.tee $t_x
        f64.const 0
        f64.max
        f64.const 1
        f64.min
        local.set $t_x

        ;; Calculate smoothstep polynomial 3t^2 - 2t^3
        local.get $t_x
        local.get $t_x
        f64.mul
        f64.const 3
        f64.mul
        local.get $t_x
        local.get $t_x
        f64.mul
        f64.const 2
        f64.mul
        f64.sub)


    ;; vec3 length
    (func $_rpu_vec3_length_f64 (param $x f64) (param $y f64) (param $z f64) (result f64)        
        local.get $x
        local.get $x
        f64.mul
        local.get $y
        local.get $y
        f64.mul
        f64.add
        local.get $z
        local.get $z
        f64.mul
        f64.add
        f64.sqrt)

    ;; function 'shader'
    (func $shader (export "shader") (param $uv_x f64) (param $uv_y f64)(param $size_x f64) (param $size_y f64) (result f64 f64 f64 f64)
        (local $d f64)
        
        local.get $uv_x
        local.get $uv_y
        
        (f64.const 0.5)
        (call $_rpu_vec2_sub_scalar_f64)
        (call $_rpu_vec2_length_f64)
        (f64.const 0.25)
        (f64.sub)
        local.set $d
        
        (f64.const 0)
        (f64.const 0.005)
        local.get $d
        
        (call $_rpu_smoothstep_vec1_f64)
        local.set $d
        local.get $d
        
        local.get $d
        
        local.get $d
        
        (f64.const 1)
        (return)
    )

    ;; function 'main'
    (func $main (export "main") (param $test f64) (result f64)
        (local $result f64)
        
        (f64.const 1)
        (f64.const 3)
        (f64.const 5)
        (call $_rpu_vec3_length_f64)
        local.set $result
        local.get $result
        
        (return)
    )
)
