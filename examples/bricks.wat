(module
    (import "env" "_rpu_sin" (func $_rpu_sin (param f64) (result f64)))
    (import "env" "_rpu_cos" (func $_rpu_cos (param f64) (result f64)))
    (import "env" "_rpu_tan" (func $_rpu_tan (param f64) (result f64)))
    (import "env" "_rpu_atan" (func $_rpu_atan (param f64) (result f64)))
    (import "env" "_rpu_radians" (func $_rpu_radians (param f64) (result f64)))
    (import "env" "_rpu_min" (func $_rpu_min (param f64) (param f64) (result f64)))
    (import "env" "_rpu_max" (func $_rpu_max (param f64) (param f64) (result f64)))
    (import "env" "_rpu_pow" (func $_rpu_pow (param f64) (param f64) (result f64)))
    (import "env" "_rpu_mod" (func $_rpu_mod (param f64) (param f64) (result f64)))
    (import "env" "_rpu_step" (func $_rpu_step (param f64) (param f64) (result f64)))
    (import "env" "_rpu_exp" (func $_rpu_exp (param f64) (result f64)))
    (import "env" "_rpu_log" (func $_rpu_log (param f64) (result f64)))
    (import "env" "_rpu_rand" (func $_rpu_rand (result f64)))
    (import "env" "_rpu_clamp" (func $_rpu_clamp (param f64) (param f64) (param f64) (result f64)))

    (memory 1)

    (global $mem_ptr (export "mem_ptr") (mut i32) (i32.const 32)) ;; We keep the first 32 bytes to shuffle stack content

    ;; Allocate memory and move the memory ptr
    (func $malloc (param $size i32) (result i32)
      (local $current_ptr i32)
      (set_local $current_ptr (global.get $mem_ptr))
      (global.set $mem_ptr
        (i32.add (get_local $current_ptr) (get_local $size)))
      (get_local $current_ptr)
    )

    ;; fract
    (func $_rpu_fract (param $x f64) (result f64)
        (f64.sub
           (local.get $x)
           (f64.floor
               (local.get $x)
           )
        )
    )
    (global $uLookAt_233_x (mut f64) (f64.const 0.4))
    (global $uLookAt_233_y (mut f64) (f64.const 0.7))
    (global $uLookAt_233_z (mut f64) (f64.const 0.0))
    (global $uOrigin_232_x (mut f64) (f64.const 0.08))
    (global $uOrigin_232_y (mut f64) (f64.const 0.5))
    (global $uOrigin_232_z (mut f64) (f64.const 2.8))

    ;; vec2 mul scalar (f64)
    (func $_rpu_vec2_mul_scalar_f64
        (param $vec2_x f64)    ;; x component of vec2
        (param $vec2_y f64)    ;; y component of vec2
        (param $scalar f64)    ;; Scalar
        (result f64 f64)       ;; Return two f64 results, the new x and y components

        ;; Calculate the new x component and return it
        (f64.mul
            (local.get $vec2_x)  ;; Get the x component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new y component and return it
        (f64.mul
            (local.get $vec2_y)  ;; Get the y component
            (local.get $scalar)  ;; Get the scalar
        )
    )

    ;; vec2 fract
    (func $_rpu_vec2_fract_f64  (param $x f64)  (param $y f64)  (result f64 f64)
        local.get $x
        (call $_rpu_fract)
        local.get $y
        (call $_rpu_fract))

    ;; vec2 floor
    (func $_rpu_vec2_floor_f64  (param $x f64)  (param $y f64)  (result f64 f64)
        local.get $x
        f64.floor
        local.get $y
        f64.floor)

    ;; vec2 mul vec2 (f64)
    (func $_rpu_vec2_mul_vec2_f64
        (param $vec2l_x f64)
        (param $vec2l_y f64)
        (param $vec2r_x f64)
        (param $vec2r_y f64)
        (result f64 f64)

        (f64.mul
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        (f64.mul
            (local.get $vec2l_y)
            (local.get $vec2r_y)
        )
    )

    ;; scalar mul vec2 (f64)
    (func $_rpu_scalar_mul_vec2_f64
        (param $scalar f64)  ;; Scalar
        (param $vec2_x f64)  ;; x component of vec2
        (param $vec2_y f64)  ;; y component of vec2
        (result f64 f64)  ;; Return two f64 results, the new x and y components

        ;; Calculate the new x component and return it
        (f64.mul
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec2_x)  ;; Get the x component
        )

        ;; Calculate the new y component and return it
        (f64.mul
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec2_y)  ;; Get the y component
        )
    )

    ;; scalar sub vec2 (f64)
    (func $_rpu_scalar_sub_vec2_f64
        (param $scalar f64)  ;; Scalar
        (param $vec2_x f64)  ;; x component of vec2
        (param $vec2_y f64)  ;; y component of vec2
        (result f64 f64)  ;; Return two f64 results, the new x and y components

        ;; Calculate the new x component and return it
        (f64.sub
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec2_x)  ;; Get the x component
        )

        ;; Calculate the new y component and return it
        (f64.sub
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec2_y)  ;; Get the y component
        )
    )

    ;; vec2 mod
    (func $_rpu_vec2_mod_f64  (param $x f64)  (param $y f64)  (param $scalar f64)  (result f64 f64)
        local.get $x
        local.get $scalar
        (call $_rpu_mod)
        local.get $y
        local.get $scalar
        (call $_rpu_mod))

    ;; vec2 dot product
    (func $_rpu_dot_product_vec2_f64  (param $a_x f64)  (param $a_y f64)  (param $b_x f64)  (param $b_y f64)  (result f64) (local $dot_product f64)
        local.get $a_x
        local.get $b_x
        f64.mul
        local.set $dot_product
        local.get $a_y
        local.get $b_y
        f64.mul
        local.get $dot_product
        f64.add)

    ;; vec1 sin
    (func $_rpu_vec1_sin_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_sin))

    ;; vec1 fract
    (func $_rpu_vec1_fract_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_fract))

    ;; vec2 add vec2 (f64)
    (func $_rpu_vec2_add_vec2_f64
        (param $vec2l_x f64)
        (param $vec2l_y f64)
        (param $vec2r_x f64)
        (param $vec2r_y f64)
        (result f64 f64)

        (f64.add
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        (f64.add
            (local.get $vec2l_y)
            (local.get $vec2r_y)
        )
    )

    ;; vec1 mix
    (func $_rpu_mix_vec1_f64 (param $edge0_x f64) (param $edge1_x f64) (param $factor f64)  (result f64 )
        
        ;; Calculate linear interpolation for component x
        local.get $edge0_x
        local.get $edge1_x
        local.get $edge0_x
        f64.sub
        local.get $factor
        f64.mul
        f64.add)

    ;; vec1 min
    (func $_rpu_vec1_min_f64  (param $x f64)  (param $scalar f64)  (result f64)
        local.get $x
        local.get $scalar
        (call $_rpu_min))

    ;; vec1 mod
    (func $_rpu_vec1_mod_f64  (param $x f64)  (param $scalar f64)  (result f64)
        local.get $x
        local.get $scalar
        (call $_rpu_mod))

    ;; vec1 floor
    (func $_rpu_vec1_floor_f64  (param $x f64)  (result f64)
        local.get $x
        f64.floor)

    ;; vec1 step
    (func $_rpu_vec1_step_f64  (param $x f64)  (param $scalar f64)  (result f64)
        local.get $x
        local.get $scalar
        (call $_rpu_step))

    ;; vec1 smoothstep
    (func $_rpu_smoothstep_vec1_f64 (param $edge0_x f64) (param $edge1_x f64) (param $x f64)  (result f64 )
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


    ;; vec1 clamp
    (func $_rpu_vec1_clamp_f64_f64  (param $x f64)  (param $scalar f64) (param $scalar2 f64) (result f64)
        local.get $x
        local.get $scalar
        local.get $scalar2
        (call $_rpu_clamp))

    ;; vec3 mul vec3 (f64)
    (func $_rpu_vec3_mul_vec3_f64
        (param $vec2l_x f64)
        (param $vec2l_y f64)
        (param $vec2l_z f64)
        (param $vec2r_x f64)
        (param $vec2r_y f64)
        (param $vec2r_z f64)
        (result f64 f64 f64)

        (f64.mul
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        (f64.mul
            (local.get $vec2l_y)
            (local.get $vec2r_y)
        )

        (f64.mul
            (local.get $vec2l_z)
            (local.get $vec2r_z)
        )
    )

    ;; vec3 mul scalar (f64)
    (func $_rpu_vec3_mul_scalar_f64
        (param $vec3_x f64)    ;; x component of vec3
        (param $vec3_y f64)    ;; y component of vec3
        (param $vec3_z f64)    ;; z component of vec3
        (param $scalar f64)    ;; Scalar
        (result f64 f64 f64)       ;; Return three f64 results, the new x, y and z components

        ;; Calculate the new x component and return it
        (f64.mul
            (local.get $vec3_x)  ;; Get the x component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new y component and return it
        (f64.mul
            (local.get $vec3_y)  ;; Get the y component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new z component and return it
        (f64.mul
            (local.get $vec3_z)  ;; Get the z component
            (local.get $scalar)  ;; Get the scalar
        )
    )

    ;; scalar sub vec3 (f64)
    (func $_rpu_scalar_sub_vec3_f64
        (param $scalar f64)  ;; Scalar
        (param $vec3_x f64)  ;; x component of vec3
        (param $vec3_y f64)  ;; y component of vec3
        (param $vec3_z f64)  ;; y component of vec3
        (result f64 f64 f64)  ;; Return three f64 results, the new x, y and z components

        ;; Calculate the new x component and return it
        (f64.sub
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec3_x)  ;; Get the x component
        )

        ;; Calculate the new y component and return it
        (f64.sub
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec3_y)  ;; Get the y component
        )

        ;; Calculate the new z component and return it
        (f64.sub
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec3_z)  ;; Get the z component
        )
    )

    ;; vec3 add vec3 (f64)
    (func $_rpu_vec3_add_vec3_f64
        (param $vec2l_x f64)
        (param $vec2l_y f64)
        (param $vec2l_z f64)
        (param $vec2r_x f64)
        (param $vec2r_y f64)
        (param $vec2r_z f64)
        (result f64 f64 f64)

        (f64.add
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        (f64.add
            (local.get $vec2l_y)
            (local.get $vec2r_y)
        )

        (f64.add
            (local.get $vec2l_z)
            (local.get $vec2r_z)
        )
    )

    ;; vec3 clamp
    (func $_rpu_vec3_clamp_f64_f64  (param $x f64)  (param $y f64)  (param $z f64)  (param $scalar f64) (param $scalar2 f64) (result f64 f64 f64)
        local.get $x
        local.get $scalar
        local.get $scalar2
        (call $_rpu_clamp)
        local.get $y
        local.get $scalar
        local.get $scalar2
        (call $_rpu_clamp)
        local.get $z
        local.get $scalar
        local.get $scalar2
        (call $_rpu_clamp))

    ;; vec3 mix
    (func $_rpu_mix_vec3_f64 (param $edge0_x f64) (param $edge0_y f64) (param $edge0_z f64) (param $edge1_x f64) (param $edge1_y f64) (param $edge1_z f64) (param $factor f64)  (result f64  f64  f64 )
        
        ;; Calculate linear interpolation for component x
        local.get $edge0_x
        local.get $edge1_x
        local.get $edge0_x
        f64.sub
        local.get $factor
        f64.mul
        f64.add
        ;; Calculate linear interpolation for component y
        local.get $edge0_y
        local.get $edge1_y
        local.get $edge0_y
        f64.sub
        local.get $factor
        f64.mul
        f64.add
        ;; Calculate linear interpolation for component z
        local.get $edge0_z
        local.get $edge1_z
        local.get $edge0_z
        f64.sub
        local.get $factor
        f64.mul
        f64.add)

    ;; vec1 neg
    (func $_rpu_vec1_neg_f64  (param $x f64)  (result f64)
        local.get $x
        f64.neg)

    ;; vec1 exp
    (func $_rpu_vec1_exp_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_exp))

    ;; vec1 log
    (func $_rpu_vec1_log_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_log))

    ;; vec1 cos
    (func $_rpu_vec1_cos_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_cos))

    ;; mat2 mul vec2 (f64)
    (func $_rpu_mat2_mul_vec2_f64
        (param $a f64)  ;; Matrix component a (row 1, col 1)
        (param $b f64)  ;; Matrix component b (row 1, col 2)
        (param $c f64)  ;; Matrix component c (row 2, col 1)
        (param $d f64)  ;; Matrix component d (row 2, col 2)
        (param $x f64)  ;; Vector component x
        (param $y f64)  ;; Vector component y
        (result f64 f64) ;; Resulting vector components

        ;; Compute the first component of the resulting vector: a*x + b*y
        local.get $a
        local.get $x
        f64.mul
        local.get $b
        local.get $y
        f64.mul
        f64.add

        ;; Compute the second component of the resulting vector: c*x + d*y
        local.get $c
        local.get $x
        f64.mul
        local.get $d
        local.get $y
        f64.mul
        f64.add
    )

    ;; vec1 max
    (func $_rpu_vec1_max_f64  (param $x f64)  (param $scalar f64)  (result f64)
        local.get $x
        local.get $scalar
        (call $_rpu_max))

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

    ;; vec2 abs
    (func $_rpu_vec2_abs_f64  (param $x f64)  (param $y f64)  (result f64 f64)
        local.get $x
        f64.abs
        local.get $y
        f64.abs)

    ;; vec2 sub vec2 (f64)
    (func $_rpu_vec2_sub_vec2_f64
        (param $vec2l_x f64)
        (param $vec2l_y f64)
        (param $vec2r_x f64)
        (param $vec2r_y f64)
        (result f64 f64)

        (f64.sub
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        (f64.sub
            (local.get $vec2l_y)
            (local.get $vec2r_y)
        )
    )

    ;; vec2 max
    (func $_rpu_vec2_max_f64  (param $x f64)  (param $y f64)  (param $scalar f64)  (result f64 f64)
        local.get $x
        local.get $scalar
        (call $_rpu_max)
        local.get $y
        local.get $scalar
        (call $_rpu_max))

    ;; vec3 abs
    (func $_rpu_vec3_abs_f64  (param $x f64)  (param $y f64)  (param $z f64)  (result f64 f64 f64)
        local.get $x
        f64.abs
        local.get $y
        f64.abs
        local.get $z
        f64.abs)

    ;; vec3 sub vec3 (f64)
    (func $_rpu_vec3_sub_vec3_f64
        (param $vec2l_x f64)
        (param $vec2l_y f64)
        (param $vec2l_z f64)
        (param $vec2r_x f64)
        (param $vec2r_y f64)
        (param $vec2r_z f64)
        (result f64 f64 f64)

        (f64.sub
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        (f64.sub
            (local.get $vec2l_y)
            (local.get $vec2r_y)
        )

        (f64.sub
            (local.get $vec2l_z)
            (local.get $vec2r_z)
        )
    )

    ;; vec3 max
    (func $_rpu_vec3_max_f64  (param $x f64)  (param $y f64)  (param $z f64)  (param $scalar f64)  (result f64 f64 f64)
        local.get $x
        local.get $scalar
        (call $_rpu_max)
        local.get $y
        local.get $scalar
        (call $_rpu_max)
        local.get $z
        local.get $scalar
        (call $_rpu_max))

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

    ;; mat4 mul vec4 (f64)
    (func $_rpu_mat4_mul_vec4_f64
        (param $a f64)  ;; Matrix component a (row 1, col 1)
        (param $b f64)  ;; Matrix component b (row 1, col 2)
        (param $c f64)  ;; Matrix component c (row 1, col 3)
        (param $d f64)  ;; Matrix component d (row 1, col 4)
        (param $e f64)  ;; Matrix component e (row 2, col 1)
        (param $f f64)  ;; Matrix component f (row 2, col 2)
        (param $g f64)  ;; Matrix component g (row 2, col 3)
        (param $h f64)  ;; Matrix component h (row 2, col 4)
        (param $i f64)  ;; Matrix component i (row 3, col 1)
        (param $j f64)  ;; Matrix component j (row 3, col 2)
        (param $k f64)  ;; Matrix component k (row 3, col 3)
        (param $l f64)  ;; Matrix component l (row 3, col 4)
        (param $m f64)  ;; Matrix component m (row 4, col 1)
        (param $n f64)  ;; Matrix component n (row 4, col 2)
        (param $o f64)  ;; Matrix component o (row 4, col 3)
        (param $p f64)  ;; Matrix component p (row 4, col 4)
        (param $x f64)  ;; Vector component x
        (param $y f64)  ;; Vector component y
        (param $z f64)  ;; Vector component z
        (param $w f64)  ;; Vector component w
        (result f64 f64 f64 f64) ;; Resulting vector components

        ;; Compute the first component of the resulting vector: a*x + b*y + c*z + d*w
        local.get $a
        local.get $x
        f64.mul
        local.get $b
        local.get $y
        f64.mul
        f64.add
        local.get $c
        local.get $z
        f64.mul
        f64.add
        local.get $d
        local.get $w
        f64.mul
        f64.add

        ;; Compute the second component of the resulting vector: e*x + f*y + g*z + h*w
        local.get $e
        local.get $x
        f64.mul
        local.get $f
        local.get $y
        f64.mul
        f64.add
        local.get $g
        local.get $z
        f64.mul
        f64.add
        local.get $h
        local.get $w
        f64.mul
        f64.add

        ;; Compute the third component of the resulting vector: i*x + j*y + k*z + l*w
        local.get $i
        local.get $x
        f64.mul
        local.get $j
        local.get $y
        f64.mul
        f64.add
        local.get $k
        local.get $z
        f64.mul
        f64.add
        local.get $l
        local.get $w
        f64.mul
        f64.add

        ;; Compute the fourth component of the resulting vector: m*x + n*y + o*z + p*w
        local.get $m
        local.get $x
        f64.mul
        local.get $n
        local.get $y
        f64.mul
        f64.add
        local.get $o
        local.get $z
        f64.mul
        f64.add
        local.get $p
        local.get $w
        f64.mul
        f64.add

    )

    ;; vec3 normalize
    (func $_rpu_normalize_vec3_f64 (param $x f64) (param $y f64) (param $z f64)  (result f64  f64  f64 )
        (local $magn f64)
         
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
        f64.sqrt
        (local.set $magn)
        local.get $x
        (local.get $magn)
        f64.div
        local.get $y
        (local.get $magn)
        f64.div
        local.get $z
        (local.get $magn)
        f64.div)

    ;; vec3 dot product
    (func $_rpu_dot_product_vec3_f64  (param $a_x f64)  (param $a_y f64)  (param $a_z f64)  (param $b_x f64)  (param $b_y f64)  (param $b_z f64)  (result f64) (local $dot_product f64)
        local.get $a_x
        local.get $b_x
        f64.mul
        local.set $dot_product
        local.get $a_y
        local.get $b_y
        f64.mul
        local.get $dot_product
        f64.add
        local.set $dot_product
        local.get $a_z
        local.get $b_z
        f64.mul
        local.get $dot_product
        f64.add)

    ;; vec1 pow
    (func $_rpu_vec1_pow_f64  (param $x f64)  (param $scalar f64)  (result f64)
        local.get $x
        local.get $scalar
        (call $_rpu_pow))

    ;; cross product
    (func $_rpu_cross_product_f64 (param $a_x f64) (param $a_y f64) (param $a_z f64) (param $b_x f64) (param $b_y f64) (param $b_z f64) (result f64 f64 f64)
        (local $c_x f64) (local $c_y f64) (local $c_z f64)
        local.get $a_y
        local.get $b_z
        f64.mul
        local.get $a_z
        local.get $b_y
        f64.mul
        f64.sub
        local.set $c_x
        local.get $a_z
        local.get $b_x
        f64.mul
        local.get $a_x
        local.get $b_z
        f64.mul
        f64.sub
        local.set $c_y
        local.get $a_x
        local.get $b_y
        f64.mul
        local.get $a_y
        local.get $b_x
        f64.mul
        f64.sub
        local.set $c_z
        local.get $c_x
        local.get $c_y
        local.get $c_z)

    ;; vec1 sqrt
    (func $_rpu_vec1_sqrt_f64  (param $x f64)  (result f64)
        local.get $x
        f64.sqrt)

    ;; vec3 div scalar (f64)
    (func $_rpu_vec3_div_scalar_f64
        (param $vec3_x f64)    ;; x component of vec3
        (param $vec3_y f64)    ;; y component of vec3
        (param $vec3_z f64)    ;; z component of vec3
        (param $scalar f64)    ;; Scalar
        (result f64 f64 f64)       ;; Return three f64 results, the new x, y and z components

        ;; Calculate the new x component and return it
        (f64.div
            (local.get $vec3_x)  ;; Get the x component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new y component and return it
        (f64.div
            (local.get $vec3_y)  ;; Get the y component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new z component and return it
        (f64.div
            (local.get $vec3_z)  ;; Get the z component
            (local.get $scalar)  ;; Get the scalar
        )
    )

    ;; vec1 atan
    (func $_rpu_vec1_atan_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_atan))

    ;; vec4 mul vec4 (f64)
    (func $_rpu_vec4_mul_vec4_f64
        (param $vec2l_x f64)
        (param $vec2l_y f64)
        (param $vec2l_z f64)
        (param $vec2l_w f64)
        (param $vec2r_x f64)
        (param $vec2r_y f64)
        (param $vec2r_z f64)
        (param $vec2r_w f64)
        (result f64 f64 f64 f64)

        (f64.mul
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        (f64.mul
            (local.get $vec2l_y)
            (local.get $vec2r_y)
        )

        (f64.mul
            (local.get $vec2l_z)
            (local.get $vec2r_z)
        )

        (f64.mul
            (local.get $vec2l_w)
            (local.get $vec2r_w)
        )
    )

    ;; vec4 mul scalar (f64)
    (func $_rpu_vec4_mul_scalar_f64
        (param $vec4_x f64)    ;; x component of vec4
        (param $vec4_y f64)    ;; y component of vec4
        (param $vec4_z f64)    ;; z component of vec4
        (param $vec4_w f64)    ;; w component of vec4
        (param $scalar f64)    ;; Scalar
        (result f64 f64 f64 f64)       ;; Return four f64 results, the new x, y, z and w components

        ;; Calculate the new x component and return it
        (f64.mul
            (local.get $vec4_x)  ;; Get the x component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new y component and return it
        (f64.mul
            (local.get $vec4_y)  ;; Get the y component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new z component and return it
        (f64.mul
            (local.get $vec4_z)  ;; Get the z component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new w component and return it
        (f64.mul
            (local.get $vec4_w)  ;; Get the w component
            (local.get $scalar)  ;; Get the scalar
        )
    )

    ;; vec4 add vec4 (f64)
    (func $_rpu_vec4_add_vec4_f64
        (param $vec2l_x f64)
        (param $vec2l_y f64)
        (param $vec2l_z f64)
        (param $vec2l_w f64)
        (param $vec2r_x f64)
        (param $vec2r_y f64)
        (param $vec2r_z f64)
        (param $vec2r_w f64)
        (result f64 f64 f64 f64)

        (f64.add
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        (f64.add
            (local.get $vec2l_y)
            (local.get $vec2r_y)
        )

        (f64.add
            (local.get $vec2l_z)
            (local.get $vec2r_z)
        )

        (f64.add
            (local.get $vec2l_w)
            (local.get $vec2r_w)
        )
    )

    ;; vec2 div vec2 (f64)
    (func $_rpu_vec2_div_vec2_f64
        (param $vec2l_x f64)
        (param $vec2l_y f64)
        (param $vec2r_x f64)
        (param $vec2r_y f64)
        (result f64 f64)

        (f64.div
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        (f64.div
            (local.get $vec2l_y)
            (local.get $vec2r_y)
        )
    )

    ;; vec1 radians
    (func $_rpu_vec1_radians_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_radians))

    ;; vec1 tan
    (func $_rpu_vec1_tan_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_tan))

    ;; scalar mul vec3 (f64)
    (func $_rpu_scalar_mul_vec3_f64
        (param $scalar f64)  ;; Scalar
        (param $vec3_x f64)  ;; x component of vec3
        (param $vec3_y f64)  ;; y component of vec3
        (param $vec3_z f64)  ;; y component of vec3
        (result f64 f64 f64)  ;; Return three f64 results, the new x, y and z components

        ;; Calculate the new x component and return it
        (f64.mul
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec3_x)  ;; Get the x component
        )

        ;; Calculate the new y component and return it
        (f64.mul
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec3_y)  ;; Get the y component
        )

        ;; Calculate the new z component and return it
        (f64.mul
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec3_z)  ;; Get the z component
        )
    )

    ;; function 'source0'
    (func $source0 (param $inPos_0_x f64) (param $inPos_0_y f64) (param $inPos_0_z f64)(param $inUV_1_x f64) (param $inUV_1_y f64)(param $inNormal_2_x f64) (param $inNormal_2_y f64) (param $inNormal_2_z f64)(param $inTime_3 f64)(param $inInput1_4_x f64) (param $inInput1_4_y f64) (param $inInput1_4_z f64)(param $inInput2_5_x f64) (param $inInput2_5_y f64) (param $inInput2_5_z f64)(param $inInput3_6_x f64) (param $inInput3_6_y f64) (param $inInput3_6_z f64) (result f64 f64 f64)
        (local $$_rpu_ternary_0_x f64)
        (local $$_rpu_ternary_0_y f64)
        (local $$_rpu_ternary_0_z f64)
        (local $p_7_x f64)
        (local $p_7_y f64)
        (local $f_8_x f64)
        (local $f_8_y f64)
        (local $h1_9_x f64)
        (local $h1_9_y f64)
        (local $h1f_10 f64)
        (local $h2_11_x f64)
        (local $h2_11_y f64)
        (local $h2f_12 f64)
        (local $h3_13_x f64)
        (local $h3_13_y f64)
        (local $h3f_14 f64)
        (local $h4_15_x f64)
        (local $h4_15_y f64)
        (local $h4f_16 f64)
        (local $noise_17 f64)

        local.get $inInput3_6_x
        (f64.const 0)
        (f64.eq)
        (if
            (then
                (f64.const 10)
                (f64.const 10)
                (f64.const 10)
                (local.set $$_rpu_ternary_0_z)
                (local.set $$_rpu_ternary_0_y)
                (local.set $$_rpu_ternary_0_x)
            )
            (else
                local.get $inInput3_6_x
                local.get $inInput3_6_y
                local.get $inInput3_6_z
                (local.set $$_rpu_ternary_0_z)
                (local.set $$_rpu_ternary_0_y)
                (local.set $$_rpu_ternary_0_x)
            )
        )
        (local.get $$_rpu_ternary_0_x)
        (local.get $$_rpu_ternary_0_y)
        (local.get $$_rpu_ternary_0_z)
        local.set $inInput3_6_z
        local.set $inInput3_6_y
        local.set $inInput3_6_x
        local.get $inUV_1_x
        local.get $inUV_1_y
        local.get $inInput3_6_x
        (call $_rpu_vec2_mul_scalar_f64)
        local.set $p_7_y
        local.set $p_7_x
        (f64.const 0)
        (f64.const 0)
        local.set $f_8_y
        local.set $f_8_x
        local.get $p_7_x
        local.get $p_7_y
        (call $_rpu_vec2_fract_f64)
        local.set $f_8_y
        local.set $f_8_x
        local.get $p_7_x
        local.get $p_7_y
        (call $_rpu_vec2_floor_f64)
        local.set $p_7_y
        local.set $p_7_x
        local.get $f_8_x
        local.get $f_8_y
        local.get $f_8_x
        local.get $f_8_y
        (call $_rpu_vec2_mul_vec2_f64)
        (f64.const 3)
        (f64.const 2)
        local.get $f_8_x
        local.get $f_8_y
        (call $_rpu_scalar_mul_vec2_f64)
        (call $_rpu_scalar_sub_vec2_f64)
        (call $_rpu_vec2_mul_vec2_f64)
        local.set $f_8_y
        local.set $f_8_x
        local.get $p_7_x
        local.get $p_7_y
        local.get $inInput3_6_x
        (call $_rpu_vec2_mod_f64)
        local.set $h1_9_y
        local.set $h1_9_x
        local.get $h1_9_x
        local.get $h1_9_y
        (f64.const 27.16898)
        (f64.const 38.90563)
        (call $_rpu_dot_product_vec2_f64)
        (call $_rpu_vec1_sin_f64)
        (f64.const 5151.5474)
        (f64.mul)
        (call $_rpu_vec1_fract_f64)
        local.set $h1f_10
        local.get $p_7_x
        local.get $p_7_y
        (f64.const 1)
        (f64.const 0)
        (call $_rpu_vec2_add_vec2_f64)
        local.get $inInput3_6_x
        (call $_rpu_vec2_mod_f64)
        local.set $h2_11_y
        local.set $h2_11_x
        local.get $h2_11_x
        local.get $h2_11_y
        (f64.const 27.16898)
        (f64.const 38.90563)
        (call $_rpu_dot_product_vec2_f64)
        (call $_rpu_vec1_sin_f64)
        (f64.const 5151.5474)
        (f64.mul)
        (call $_rpu_vec1_fract_f64)
        local.set $h2f_12
        local.get $p_7_x
        local.get $p_7_y
        (f64.const 0)
        (f64.const 1)
        (call $_rpu_vec2_add_vec2_f64)
        local.get $inInput3_6_x
        (call $_rpu_vec2_mod_f64)
        local.set $h3_13_y
        local.set $h3_13_x
        local.get $h3_13_x
        local.get $h3_13_y
        (f64.const 27.16898)
        (f64.const 38.90563)
        (call $_rpu_dot_product_vec2_f64)
        (call $_rpu_vec1_sin_f64)
        (f64.const 5151.5474)
        (f64.mul)
        (call $_rpu_vec1_fract_f64)
        local.set $h3f_14
        local.get $p_7_x
        local.get $p_7_y
        (f64.const 1)
        (f64.const 1)
        (call $_rpu_vec2_add_vec2_f64)
        local.get $inInput3_6_x
        (call $_rpu_vec2_mod_f64)
        local.set $h4_15_y
        local.set $h4_15_x
        local.get $h4_15_x
        local.get $h4_15_y
        (f64.const 27.16898)
        (f64.const 38.90563)
        (call $_rpu_dot_product_vec2_f64)
        (call $_rpu_vec1_sin_f64)
        (f64.const 5151.5474)
        (f64.mul)
        (call $_rpu_vec1_fract_f64)
        local.set $h4f_16
        local.get $h1f_10
        local.get $h2f_12
        local.get $f_8_x
        (call $_rpu_mix_vec1_f64)
        local.get $h3f_14
        local.get $h4f_16
        local.get $f_8_x
        (call $_rpu_mix_vec1_f64)
        local.get $f_8_y
        (call $_rpu_mix_vec1_f64)
        local.set $noise_17
        local.get $noise_17
        local.get $noise_17
        local.get $noise_17
        local.set $inInput1_4_z
        local.set $inInput1_4_y
        local.set $inInput1_4_x
        local.get $inInput1_4_x
        local.get $inInput1_4_y
        local.get $inInput1_4_z
        (return)
    )

    ;; function 'source1'
    (func $source1 (param $inPos_18_x f64) (param $inPos_18_y f64) (param $inPos_18_z f64)(param $inUV_19_x f64) (param $inUV_19_y f64)(param $inNormal_20_x f64) (param $inNormal_20_y f64) (param $inNormal_20_z f64)(param $inTime_21 f64)(param $inInput1_22_x f64) (param $inInput1_22_y f64) (param $inInput1_22_z f64)(param $inInput2_23_x f64) (param $inInput2_23_y f64) (param $inInput2_23_z f64)(param $inInput3_24_x f64) (param $inInput3_24_y f64) (param $inInput3_24_z f64) (result f64 f64 f64)
        (local $scale_25 f64)
        (local $f_26 f64)
        (local $p_27_x f64)
        (local $p_27_y f64)
        (local $amp_28 f64)
        (local $i_29 i64)
        (local $_rpu_temp_f64 f64)
        (f64.const 20)
        local.set $scale_25
        (f64.const 0)
        local.set $f_26
        local.get $inUV_19_x
        local.get $inUV_19_y
        local.get $scale_25
        (call $_rpu_vec2_mod_f64)
        local.set $p_27_y
        local.set $p_27_x
        (f64.const 0.6)
        local.set $amp_28

        (i64.const 0)
        local.set $i_29
        (block
            (loop
                local.get $i_29
                (i64.const 5)
                (i64.lt_s)
                (i32.eqz)
                (br_if 1)
                (block
                    local.get $inPos_18_x
                    local.get $inPos_18_y
                    local.get $inPos_18_z
                    local.get $p_27_x
                    local.get $p_27_y
                    local.get $inNormal_20_x
                    local.get $inNormal_20_y
                    local.get $inNormal_20_z
                    local.get $inTime_21
                    local.get $inInput1_22_x
                    local.get $inInput1_22_y
                    local.get $inInput1_22_z
                    local.get $inInput2_23_x
                    local.get $inInput2_23_y
                    local.get $inInput2_23_z
                    local.get $scale_25
                    local.get $scale_25
                    local.get $scale_25
                    (call $source0)
                    (local.set $_rpu_temp_f64)
                    (i32.const 16)
                    (local.get $_rpu_temp_f64)
                    (f64.store)
                    (local.set $_rpu_temp_f64)
                    (i32.const 8)
                    (local.get $_rpu_temp_f64)
                    (f64.store)
                    (local.set $_rpu_temp_f64)
                    (i32.const 0)
                    (local.get $_rpu_temp_f64)
                    (f64.store)
                    (i32.const 0)
                    (f64.load)
                    local.get $amp_28
                    (f64.mul)
                    local.get $f_26
                    f64.add
                    local.set $f_26
                    (f64.const 0.5)
                    local.get $amp_28
                    f64.mul
                    local.set $amp_28
                    (f64.const 2)
                    local.get $scale_25
                    f64.mul
                    local.set $scale_25
                )
                (i64.const 1)
                local.get $i_29
                i64.add
                local.set $i_29
                (br 0)
            )
        )
        local.get $f_26
        (f64.const 1)
        (call $_rpu_vec1_min_f64)
        local.get $f_26
        (f64.const 1)
        (call $_rpu_vec1_min_f64)
        local.get $f_26
        (f64.const 1)
        (call $_rpu_vec1_min_f64)
        local.set $inInput1_22_z
        local.set $inInput1_22_y
        local.set $inInput1_22_x
        local.get $inInput1_22_x
        local.get $inInput1_22_y
        local.get $inInput1_22_z
        (return)
    )

    ;; function 'source2'
    (func $source2 (param $inPos_30_x f64) (param $inPos_30_y f64) (param $inPos_30_z f64)(param $inUV_31_x f64) (param $inUV_31_y f64)(param $inNormal_32_x f64) (param $inNormal_32_y f64) (param $inNormal_32_z f64)(param $inTime_33 f64)(param $inInput1_34_x f64) (param $inInput1_34_y f64) (param $inInput1_34_z f64)(param $inInput2_35_x f64) (param $inInput2_35_y f64) (param $inInput2_35_z f64)(param $inInput3_36_x f64) (param $inInput3_36_y f64) (param $inInput3_36_z f64) (result f64 f64 f64)
        (local $BMWIDTH_37 f64)
        (local $BMHEIGHT_38 f64)
        (local $MWF_39 f64)
        (local $MHF_40 f64)
        (local $ss_41 f64)
        (local $tt_42 f64)
        (local $sbrick_43 f64)
        (local $tbrick_44 f64)
        (local $randv2_45_x f64)
        (local $randv2_45_y f64)
        (local $noise_46 f64)
        (local $w_47 f64)
        (local $h_48 f64)
        (local $sbump_49 f64)
        (local $tbump_50 f64)
        (local $bump_51 f64)
        (local $unevenBump_52 f64)
        (f64.const 0.25)
        (f64.const 0.03)
        (f64.add)
        local.set $BMWIDTH_37
        (f64.const 0.08)
        (f64.const 0.03)
        (f64.add)
        local.set $BMHEIGHT_38
        (f64.const 0.03)
        (f64.const 0.5)
        (f64.mul)
        local.get $BMWIDTH_37
        (f64.div)
        local.set $MWF_39
        (f64.const 0.03)
        (f64.const 0.5)
        (f64.mul)
        local.get $BMHEIGHT_38
        (f64.div)
        local.set $MHF_40
        local.get $inUV_31_x
        local.get $BMWIDTH_37
        (f64.div)
        local.set $ss_41
        local.get $inUV_31_y
        local.get $BMHEIGHT_38
        (f64.div)
        local.set $tt_42

        local.get $tt_42
        (f64.const 0.5)
        (f64.mul)
        (f64.const 1)
        (call $_rpu_vec1_mod_f64)
        (f64.const 0.5)
        (f64.gt)
        (if
            (then
                local.get $ss_41
                (f64.const 0.5)
                (f64.add)
                local.set $ss_41
            )
        )
        local.get $ss_41
        (call $_rpu_vec1_floor_f64)
        local.set $sbrick_43
        local.get $tt_42
        (call $_rpu_vec1_floor_f64)
        local.set $tbrick_44
        (call $_rpu_rand)
        (call $_rpu_rand)
        local.set $randv2_45_y
        local.set $randv2_45_x
        local.get $randv2_45_x
        local.get $randv2_45_y
        (f64.const 12.9898)
        (f64.const 78.233)
        (call $_rpu_dot_product_vec2_f64)
        (call $_rpu_vec1_sin_f64)
        (f64.const 43758.547)
        (f64.mul)
        (call $_rpu_vec1_fract_f64)
        local.set $noise_46
        local.get $ss_41
        local.get $sbrick_43
        (f64.sub)
        local.set $ss_41
        local.get $tt_42
        local.get $tbrick_44
        (f64.sub)
        local.set $tt_42
        local.get $MWF_39
        local.get $ss_41
        (call $_rpu_vec1_step_f64)
        (f64.const 1)
        local.get $MWF_39
        (f64.sub)
        local.get $ss_41
        (call $_rpu_vec1_step_f64)
        (f64.sub)
        local.set $w_47
        local.get $MHF_40
        local.get $tt_42
        (call $_rpu_vec1_step_f64)
        (f64.const 1)
        local.get $MHF_40
        (f64.sub)
        local.get $tt_42
        (call $_rpu_vec1_step_f64)
        (f64.sub)
        local.set $h_48
        (f64.const 0)
        local.get $MWF_39
        local.get $ss_41
        (call $_rpu_smoothstep_vec1_f64)
        (f64.const 1)
        local.get $MWF_39
        (f64.sub)
        (f64.const 1)
        local.get $ss_41
        (call $_rpu_smoothstep_vec1_f64)
        (f64.sub)
        local.set $sbump_49
        (f64.const 0)
        local.get $MHF_40
        local.get $tt_42
        (call $_rpu_smoothstep_vec1_f64)
        (f64.const 1)
        local.get $MHF_40
        (f64.sub)
        (f64.const 1)
        local.get $tt_42
        (call $_rpu_smoothstep_vec1_f64)
        (f64.sub)
        local.set $tbump_50
        local.get $sbump_49
        local.get $tbump_50
        (f64.mul)
        local.set $bump_51
        local.get $bump_51
        local.set $unevenBump_52

        local.get $noise_46
        (f64.const 0.25)
        (f64.le)
        (if
            (then
                local.get $ss_41
                (f64.const 1)
                (f64.mul)
                local.get $unevenBump_52
                f64.sub
                local.set $unevenBump_52
            )
            (else

                local.get $noise_46
                (f64.const 0.5)
                (f64.le)
                (if
                    (then
                        local.get $tt_42
                        (f64.const 1)
                        (f64.mul)
                        local.get $unevenBump_52
                        f64.sub
                        local.set $unevenBump_52
                    )
                    (else

                        local.get $noise_46
                        (f64.const 0.75)
                        (f64.le)
                        (if
                            (then
                                local.get $BMWIDTH_37
                                local.get $ss_41
                                (f64.sub)
                                (f64.const 1)
                                (f64.mul)
                                local.get $unevenBump_52
                                f64.sub
                                local.set $unevenBump_52
                            )
                            (else
                                local.get $BMHEIGHT_38
                                local.get $tt_42
                                (f64.sub)
                                (f64.const 1)
                                (f64.mul)
                                local.get $unevenBump_52
                                f64.sub
                                local.set $unevenBump_52
                            )
                        )
                    )
                )
            )
        )
        local.get $noise_46
        local.get $bump_51
        local.get $unevenBump_52
        (f64.const 0)
        (f64.const 1)
        (call $_rpu_vec1_clamp_f64_f64)
        local.set $inInput1_34_z
        local.set $inInput1_34_y
        local.set $inInput1_34_x
        local.get $inInput1_34_x
        local.get $inInput1_34_y
        local.get $inInput1_34_z
        (return)
    )

    ;; function 'Kogj'
    (func $Kogj (param $pos_53_x f64) (param $pos_53_y f64) (param $pos_53_z f64)(param $uv_54_x f64) (param $uv_54_y f64)(param $normal_55_x f64) (param $normal_55_y f64) (param $normal_55_z f64)(param $time_56 f64)(param $param_57_x f64) (param $param_57_y f64) (param $param_57_z f64) (result f64 f64 f64)
        (local $_Zxur_58_x f64)
        (local $_Zxur_58_y f64)
        (local $_Zxur_58_z f64)
        (local $_Eoks_59_x f64)
        (local $_Eoks_59_y f64)
        (local $_Eoks_59_z f64)
        local.get $pos_53_x
        local.get $pos_53_y
        local.get $pos_53_z
        local.get $uv_54_x
        local.get $uv_54_y
        local.get $normal_55_x
        local.get $normal_55_y
        local.get $normal_55_z
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (call $source0)
        local.set $_Zxur_58_z
        local.set $_Zxur_58_y
        local.set $_Zxur_58_x
        local.get $pos_53_x
        local.get $pos_53_y
        local.get $pos_53_z
        local.get $uv_54_x
        local.get $uv_54_y
        local.get $normal_55_x
        local.get $normal_55_y
        local.get $normal_55_z
        (f64.const 0)
        local.get $_Zxur_58_x
        local.get $_Zxur_58_y
        local.get $_Zxur_58_z
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (call $source1)
        local.set $_Eoks_59_z
        local.set $_Eoks_59_y
        local.set $_Eoks_59_x
        local.get $_Eoks_59_x
        local.get $_Eoks_59_y
        local.get $_Eoks_59_z
        (return)
    )

    ;; function 'dhhB'
    (func $dhhB (param $pos_60_x f64) (param $pos_60_y f64) (param $pos_60_z f64)(param $uv_61_x f64) (param $uv_61_y f64)(param $normal_62_x f64) (param $normal_62_y f64) (param $normal_62_z f64)(param $time_63 f64)(param $param_64_x f64) (param $param_64_y f64) (param $param_64_z f64) (result f64 f64 f64)
        (local $_BdHg_65_x f64)
        (local $_BdHg_65_y f64)
        (local $_BdHg_65_z f64)
        (local $_bpea_66_x f64)
        (local $_bpea_66_y f64)
        (local $_bpea_66_z f64)
        (local $_eukA_67_x f64)
        (local $_eukA_67_y f64)
        (local $_eukA_67_z f64)
        (local $_rpu_temp_f64 f64)
        (local $_xeHM_68_x f64)
        (local $_xeHM_68_y f64)
        (local $_xeHM_68_z f64)
        (local $_jIHy_69_x f64)
        (local $_jIHy_69_y f64)
        (local $_jIHy_69_z f64)
        (local $_vRSx_70_x f64)
        (local $_vRSx_70_y f64)
        (local $_vRSx_70_z f64)
        (local $_sRul_71_x f64)
        (local $_sRul_71_y f64)
        (local $_sRul_71_z f64)
        (local $_YTpn_72_x f64)
        (local $_YTpn_72_y f64)
        (local $_YTpn_72_z f64)
        (local $_xzCJ_73_x f64)
        (local $_xzCJ_73_y f64)
        (local $_xzCJ_73_z f64)
        (local $_vTdO_74_x f64)
        (local $_vTdO_74_y f64)
        (local $_vTdO_74_z f64)
        (local $_bzqA_75_x f64)
        (local $_bzqA_75_y f64)
        (local $_bzqA_75_z f64)
        (local $_gTOo_76_x f64)
        (local $_gTOo_76_y f64)
        (local $_gTOo_76_z f64)
        (local $_jkHH_77_x f64)
        (local $_jkHH_77_y f64)
        (local $_jkHH_77_z f64)
        (local $_MxCk_78_x f64)
        (local $_MxCk_78_y f64)
        (local $_MxCk_78_z f64)
        (local $_PZGo_79_x f64)
        (local $_PZGo_79_y f64)
        (local $_PZGo_79_z f64)
        (local $_fTFt_80_x f64)
        (local $_fTFt_80_y f64)
        (local $_fTFt_80_z f64)
        local.get $param_64_x
        local.get $param_64_y
        local.get $param_64_z
        (f64.const 1)
        (f64.const 1)
        (f64.const 1)
        (call $_rpu_vec3_mul_vec3_f64)
        local.set $_BdHg_65_z
        local.set $_BdHg_65_y
        local.set $_BdHg_65_x
        local.get $_BdHg_65_y
        local.get $_BdHg_65_y
        local.get $_BdHg_65_y
        local.set $_bpea_66_z
        local.set $_bpea_66_y
        local.set $_bpea_66_x
        local.get $_BdHg_65_z
        local.get $_BdHg_65_z
        local.get $_BdHg_65_z
        local.set $_eukA_67_z
        local.set $_eukA_67_y
        local.set $_eukA_67_x
        local.get $_eukA_67_x
        local.get $_eukA_67_y
        local.get $_eukA_67_z
        local.get $uv_61_x
        local.get $uv_61_y
        (f64.const 0)
        (local.set $_rpu_temp_f64)
        (i32.const 16)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 8)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 0)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (i32.const 0)
        (f64.load)
        (i32.const 8)
        (f64.load)
        local.get $normal_62_x
        local.get $normal_62_y
        local.get $normal_62_z
        local.get $time_63
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (call $Kogj)
        local.set $_xeHM_68_z
        local.set $_xeHM_68_y
        local.set $_xeHM_68_x
        local.get $_xeHM_68_x
        local.get $_xeHM_68_y
        local.get $_xeHM_68_z
        local.get $_eukA_67_x
        local.get $_eukA_67_y
        local.get $_eukA_67_z
        (call $_rpu_vec3_mul_vec3_f64)
        (f64.const 1)
        (f64.const 1)
        (f64.const 1)
        (call $_rpu_vec3_mul_vec3_f64)
        local.set $_jIHy_69_z
        local.set $_jIHy_69_y
        local.set $_jIHy_69_x
        local.get $uv_61_x
        (f64.const 1)
        (f64.const 1)
        (local.set $_rpu_temp_f64)
        (i32.const 8)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 0)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (i32.const 0)
        (f64.load)
        (f64.mul)
        local.get $uv_61_y
        (f64.const 1)
        (f64.const 1)
        (local.set $_rpu_temp_f64)
        (i32.const 8)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 0)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (i32.const 8)
        (f64.load)
        (f64.mul)
        (f64.const 1)
        local.set $_vRSx_70_z
        local.set $_vRSx_70_y
        local.set $_vRSx_70_x
        local.get $_vRSx_70_x
        local.get $_vRSx_70_y
        local.get $_vRSx_70_z
        (f64.const 3)
        (call $_rpu_vec3_mul_scalar_f64)
        (f64.const 1)
        (f64.const 1)
        (f64.const 1)
        (call $_rpu_vec3_mul_vec3_f64)
        local.set $_sRul_71_z
        local.set $_sRul_71_y
        local.set $_sRul_71_x
        local.get $_sRul_71_x
        local.get $_sRul_71_y
        local.get $_sRul_71_z
        local.get $_sRul_71_x
        local.get $_sRul_71_y
        local.get $normal_62_x
        local.get $normal_62_y
        local.get $normal_62_z
        local.get $time_63
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (call $Kogj)
        local.set $_YTpn_72_z
        local.set $_YTpn_72_y
        local.set $_YTpn_72_x
        local.get $_YTpn_72_x
        local.get $_YTpn_72_y
        local.get $_YTpn_72_z
        (f64.const 7.845)
        (call $_rpu_vec3_mul_scalar_f64)
        (f64.const 1)
        (f64.const 1)
        (f64.const 1)
        (call $_rpu_vec3_mul_vec3_f64)
        local.set $_xzCJ_73_z
        local.set $_xzCJ_73_y
        local.set $_xzCJ_73_x
        (f64.const 1)
        local.get $_bpea_66_x
        local.get $_bpea_66_y
        local.get $_bpea_66_z
        (call $_rpu_scalar_sub_vec3_f64)
        local.set $_vTdO_74_z
        local.set $_vTdO_74_y
        local.set $_vTdO_74_x
        local.get $_vTdO_74_x
        local.get $_vTdO_74_y
        local.get $_vTdO_74_z
        (f64.const 0.1)
        (call $_rpu_vec3_mul_scalar_f64)
        (f64.const 1)
        (f64.const 1)
        (f64.const 1)
        (call $_rpu_vec3_mul_vec3_f64)
        local.set $_bzqA_75_z
        local.set $_bzqA_75_y
        local.set $_bzqA_75_x
        local.get $_bzqA_75_x
        local.get $_bzqA_75_y
        local.get $_bzqA_75_z
        local.get $_bzqA_75_x
        local.get $_bzqA_75_y
        local.get $_bzqA_75_z
        (call $_rpu_vec3_mul_vec3_f64)
        local.get $_xzCJ_73_x
        local.get $_xzCJ_73_y
        local.get $_xzCJ_73_z
        (call $_rpu_vec3_mul_vec3_f64)
        local.set $_gTOo_76_z
        local.set $_gTOo_76_y
        local.set $_gTOo_76_x
        local.get $_gTOo_76_x
        local.get $_gTOo_76_y
        local.get $_gTOo_76_z
        (f64.const 5)
        (call $_rpu_vec3_mul_scalar_f64)
        (f64.const 1)
        (f64.const 1)
        (f64.const 1)
        (call $_rpu_vec3_mul_vec3_f64)
        local.set $_jkHH_77_z
        local.set $_jkHH_77_y
        local.set $_jkHH_77_x
        local.get $_jkHH_77_x
        local.get $_jkHH_77_y
        local.get $_jkHH_77_z
        local.get $_jIHy_69_x
        local.get $_jIHy_69_y
        local.get $_jIHy_69_z
        (call $_rpu_vec3_add_vec3_f64)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (call $_rpu_vec3_add_vec3_f64)
        local.set $_MxCk_78_z
        local.set $_MxCk_78_y
        local.set $_MxCk_78_x
        local.get $_MxCk_78_x
        local.get $_MxCk_78_y
        local.get $_MxCk_78_z
        (f64.const 1.6)
        (call $_rpu_vec3_mul_scalar_f64)
        (f64.const 1)
        (f64.const 1)
        (f64.const 1)
        (call $_rpu_vec3_mul_vec3_f64)
        local.set $_PZGo_79_z
        local.set $_PZGo_79_y
        local.set $_PZGo_79_x
        local.get $_PZGo_79_x
        local.get $_PZGo_79_y
        local.get $_PZGo_79_z
        (f64.const 0)
        (f64.const 1.1)
        (call $_rpu_vec3_clamp_f64_f64)
        local.set $_fTFt_80_z
        local.set $_fTFt_80_y
        local.set $_fTFt_80_x
        local.get $_fTFt_80_x
        local.get $_fTFt_80_y
        local.get $_fTFt_80_z
        (return)
    )

    ;; function 'material0'
    (func $material0 (param $pos_81_x f64) (param $pos_81_y f64) (param $pos_81_z f64)(param $normal_82_x f64) (param $normal_82_y f64) (param $normal_82_z f64)(param $time_83 f64)(param $material_84 i32) 
        (local $uv_85_x f64)
        (local $uv_85_y f64)
        (local $_rpu_temp_f64 f64)
        (local $_oCzV_86_x f64)
        (local $_oCzV_86_y f64)
        (local $_oCzV_86_z f64)
        (local $_FCZn_87_x f64)
        (local $_FCZn_87_y f64)
        (local $_FCZn_87_z f64)
        (local $_UaEB_88_x f64)
        (local $_UaEB_88_y f64)
        (local $_UaEB_88_z f64)
        (local $_khBL_89_x f64)
        (local $_khBL_89_y f64)
        (local $_khBL_89_z f64)
        (local $_jZEF_90_x f64)
        (local $_jZEF_90_y f64)
        (local $_jZEF_90_z f64)
        (local $_IiAx_91_x f64)
        (local $_IiAx_91_y f64)
        (local $_IiAx_91_z f64)
        (local $_eaWS_92_x f64)
        (local $_eaWS_92_y f64)
        (local $_eaWS_92_z f64)
        (local $_ylZT_93_x f64)
        (local $_ylZT_93_y f64)
        (local $_ylZT_93_z f64)
        (f64.const 0)
        (f64.const 0)
        local.set $uv_85_y
        local.set $uv_85_x
        (f64.const 1)
        (f64.const 1)
        (f64.const 1)
        (local.set $_rpu_temp_f64)
        (local.get $material_84)
        (i32.const 48)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $material_84)
        (i32.const 40)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $material_84)
        (i32.const 32)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (f64.const 0)
        (local.set $_rpu_temp_f64)
        (local.get $material_84)
        (i32.const 56)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (f64.const 0)
        (local.set $_rpu_temp_f64)
        (local.get $material_84)
        (i32.const 64)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (f64.const 1)
        (local.set $_rpu_temp_f64)
        (local.get $material_84)
        (i32.const 80)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        local.get $pos_81_x
        local.get $pos_81_y
        local.set $uv_85_y
        local.set $uv_85_x
        (f64.const 0.5)
        (local.set $_rpu_temp_f64)
        (local.get $material_84)
        (i32.const 88)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        local.get $pos_81_x
        local.get $pos_81_y
        local.get $pos_81_z
        local.get $uv_85_x
        local.get $uv_85_y
        local.get $normal_82_x
        local.get $normal_82_y
        local.get $normal_82_z
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (call $source2)
        local.set $_oCzV_86_z
        local.set $_oCzV_86_y
        local.set $_oCzV_86_x
        local.get $_oCzV_86_y
        local.get $_oCzV_86_y
        local.get $_oCzV_86_y
        local.set $_FCZn_87_z
        local.set $_FCZn_87_y
        local.set $_FCZn_87_x
        local.get $_FCZn_87_x
        local.get $_FCZn_87_y
        local.get $_FCZn_87_z
        (f64.const 0.46)
        (call $_rpu_vec3_mul_scalar_f64)
        (f64.const 1)
        (f64.const 1)
        (f64.const 1)
        (call $_rpu_vec3_mul_vec3_f64)
        local.set $_UaEB_88_z
        local.set $_UaEB_88_y
        local.set $_UaEB_88_x
        local.get $_oCzV_86_x
        local.get $_oCzV_86_x
        local.get $_oCzV_86_x
        local.set $_khBL_89_z
        local.set $_khBL_89_y
        local.set $_khBL_89_x
        local.get $_khBL_89_x
        local.get $_khBL_89_y
        local.get $_khBL_89_z
        local.get $uv_85_x
        local.get $uv_85_y
        (f64.const 0)
        (local.set $_rpu_temp_f64)
        (i32.const 16)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 8)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 0)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (i32.const 0)
        (f64.load)
        (i32.const 8)
        (f64.load)
        local.get $normal_82_x
        local.get $normal_82_y
        local.get $normal_82_z
        local.get $time_83
        local.get $_oCzV_86_x
        local.get $_oCzV_86_y
        local.get $_oCzV_86_z
        (call $dhhB)
        local.set $_jZEF_90_z
        local.set $_jZEF_90_y
        local.set $_jZEF_90_x
        (f64.const 0.521)
        (f64.const 0.21)
        (f64.const 0.21)
        (f64.const 0.872)
        (f64.const 0.205)
        (f64.const 0.223)
        local.get $_khBL_89_x
        (call $_rpu_mix_vec3_f64)
        local.set $_IiAx_91_z
        local.set $_IiAx_91_y
        local.set $_IiAx_91_x
        (f64.const 0.86)
        (f64.const 0.73)
        (f64.const 0.489)
        (local.set $_rpu_temp_f64)
        (i32.const 16)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 8)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 0)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (i32.const 0)
        (f64.load)
        (i32.const 8)
        (f64.load)
        (i32.const 16)
        (f64.load)
        local.set $_eaWS_92_z
        local.set $_eaWS_92_y
        local.set $_eaWS_92_x
        local.get $_eaWS_92_x
        local.get $_eaWS_92_y
        local.get $_eaWS_92_z
        local.get $_IiAx_91_x
        local.get $_IiAx_91_y
        local.get $_IiAx_91_z
        local.get $_FCZn_87_x
        (call $_rpu_mix_vec3_f64)
        local.set $_ylZT_93_z
        local.set $_ylZT_93_y
        local.set $_ylZT_93_x
        local.get $_UaEB_88_x
        (local.set $_rpu_temp_f64)
        (local.get $material_84)
        (i32.const 72)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        local.get $_jZEF_90_x
        (local.set $_rpu_temp_f64)
        (local.get $material_84)
        (i32.const 88)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        local.get $_ylZT_93_x
        local.get $_ylZT_93_y
        local.get $_ylZT_93_z
        (local.set $_rpu_temp_f64)
        (local.get $material_84)
        (i32.const 24)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $material_84)
        (i32.const 16)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $material_84)
        (i32.const 8)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
    )

    ;; function 'smin'
    (func $smin (param $a_94 f64)(param $b_95 f64)(param $k_96 f64) (result f64)
        (local $res_97 f64)
        local.get $k_96
        (call $_rpu_vec1_neg_f64)
        local.get $a_94
        (f64.mul)
        (call $_rpu_vec1_exp_f64)
        local.get $k_96
        (call $_rpu_vec1_neg_f64)
        local.get $b_95
        (f64.mul)
        (call $_rpu_vec1_exp_f64)
        (f64.add)
        local.set $res_97
        local.get $res_97
        (call $_rpu_vec1_log_f64)
        (call $_rpu_vec1_neg_f64)
        local.get $k_96
        (f64.div)
        (return)
    )

    ;; function 'opBlend'
    (func $opBlend (param $d1_98_x f64) (param $d1_98_y f64) (param $d1_98_z f64)(param $d2_99_x f64) (param $d2_99_y f64) (param $d2_99_z f64)(param $k_100 f64) (result f64 f64 f64)
        (local $rc_101_x f64)
        (local $rc_101_y f64)
        (local $rc_101_z f64)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $rc_101_z
        local.set $rc_101_y
        local.set $rc_101_x
        local.get $d1_98_x
        local.get $d2_99_x
        local.get $k_100
        (call $smin)
        local.set $rc_101_x

        local.get $d1_98_x
        local.get $d2_99_x
        (f64.lt)
        (if
            (then
                (block
                    local.get $d1_98_y
                    local.set $rc_101_y
                    local.get $d1_98_z
                    local.set $rc_101_z
                )
            )
            (else
                (block
                    local.get $d2_99_y
                    local.set $rc_101_y
                    local.get $d2_99_z
                    local.set $rc_101_z
                )
            )
        )
        local.get $rc_101_x
        local.get $rc_101_y
        local.get $rc_101_z
        (return)
    )

    ;; function 'opU'
    (func $opU (param $d1_102_x f64) (param $d1_102_y f64) (param $d1_102_z f64)(param $d2_103_x f64) (param $d2_103_y f64) (param $d2_103_z f64) (result f64 f64 f64)
        (local $$_rpu_ternary_1_x f64)
        (local $$_rpu_ternary_1_y f64)
        (local $$_rpu_ternary_1_z f64)

        local.get $d1_102_x
        local.get $d2_103_x
        (f64.lt)
        (if
            (then
                local.get $d1_102_x
                local.get $d1_102_y
                local.get $d1_102_z
                (local.set $$_rpu_ternary_1_z)
                (local.set $$_rpu_ternary_1_y)
                (local.set $$_rpu_ternary_1_x)
            )
            (else
                local.get $d2_103_x
                local.get $d2_103_y
                local.get $d2_103_z
                (local.set $$_rpu_ternary_1_z)
                (local.set $$_rpu_ternary_1_y)
                (local.set $$_rpu_ternary_1_x)
            )
        )
        (local.get $$_rpu_ternary_1_x)
        (local.get $$_rpu_ternary_1_y)
        (local.get $$_rpu_ternary_1_z)
        (return)
    )

    ;; function 'opTwist'
    (func $opTwist (param $p_104_x f64) (param $p_104_y f64) (param $p_104_z f64)(param $twist_105 f64) (result f64 f64 f64)
        (local $c_106 f64)
        (local $s_107 f64)
        (local $m_108_x f64)
        (local $m_108_y f64)
        (local $m_108_z f64)
        (local $m_108_w f64)
        (local $q_109_x f64)
        (local $q_109_y f64)
        (local $q_109_z f64)
        local.get $twist_105
        local.get $p_104_z
        (f64.mul)
        (call $_rpu_vec1_cos_f64)
        local.set $c_106
        local.get $twist_105
        local.get $p_104_z
        (f64.mul)
        (call $_rpu_vec1_sin_f64)
        local.set $s_107
        local.get $c_106
        local.get $s_107
        (call $_rpu_vec1_neg_f64)
        local.get $s_107
        local.get $c_106
        (local.set $m_108_w)
        (local.set $m_108_z)
        (local.set $m_108_y)
        (local.set $m_108_x)
        (local.get $m_108_x)
        (local.get $m_108_y)
        (local.get $m_108_z)
        (local.get $m_108_w)
        local.get $p_104_x
        local.get $p_104_y
        (call $_rpu_mat2_mul_vec2_f64)
        local.get $p_104_z
        local.set $q_109_z
        local.set $q_109_y
        local.set $q_109_x
        local.get $q_109_x
        local.get $q_109_y
        local.get $q_109_z
        (return)
    )

    ;; function 'opS'
    (func $opS (param $d1_110 f64)(param $d2_111 f64) (result f64)
        local.get $d2_111
        (call $_rpu_vec1_neg_f64)
        local.get $d1_110
        (call $_rpu_vec1_max_f64)
        (return)
    )

    ;; function 'sdCylinder'
    (func $sdCylinder (param $p_112_x f64) (param $p_112_y f64) (param $p_112_z f64)(param $h_113_x f64) (param $h_113_y f64) (result f64)
        (local $d_114_x f64)
        (local $d_114_y f64)
        local.get $p_112_x
        local.get $p_112_z
        (call $_rpu_vec2_length_f64)
        local.get $p_112_y
        (call $_rpu_vec2_abs_f64)
        local.get $h_113_x
        local.get $h_113_y
        (call $_rpu_vec2_sub_vec2_f64)
        local.set $d_114_y
        local.set $d_114_x
        local.get $d_114_x
        local.get $d_114_y
        (call $_rpu_vec1_max_f64)
        (f64.const 0)
        (call $_rpu_vec1_min_f64)
        local.get $d_114_x
        local.get $d_114_y
        (f64.const 0)
        (call $_rpu_vec2_max_f64)
        (call $_rpu_vec2_length_f64)
        (f64.add)
        (return)
    )

    ;; function 'random'
    (func $random  (result f64)
        (call $_rpu_rand)
        (return)
    )

    ;; function 'rand2'
    (func $rand2  (result f64 f64)
        (call $_rpu_rand)
        (call $_rpu_rand)
        (return)
    )

    ;; function 'map'
    (func $map (param $p_115_x f64) (param $p_115_y f64) (param $p_115_z f64) (result f64 f64 f64)
        (local $res_116_x f64)
        (local $res_116_y f64)
        (local $res_116_z f64)
        (local $tp_118_x f64)
        (local $tp_118_y f64)
        (local $tp_118_z f64)
        (local $temp_119_x f64)
        (local $temp_119_y f64)
        (local $temp_119_z f64)
        (local $gResult1_120_x f64)
        (local $gResult1_120_y f64)
        (local $gResult1_120_z f64)
        (local $gResult2_121_x f64)
        (local $gResult2_121_y f64)
        (local $gResult2_121_z f64)
        (local $_rpu_temp_f64 f64)
        (local $bumpNormal_122_x f64)
        (local $bumpNormal_122_y f64)
        (local $bumpNormal_122_z f64)
        (local $_rpu_temp_mem_ptr i32)
        (local $_rpu_temp_i64 i64)
        (local $bumpMaterial_123 i32)
        (f64.const 1000000)
        (f64.const 2)
        (call $_rpu_vec1_neg_f64)
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        local.set $res_116_z
        local.set $res_116_y
        local.set $res_116_x
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $temp_119_z
        local.set $temp_119_y
        local.set $temp_119_x
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $gResult1_120_z
        local.set $gResult1_120_y
        local.set $gResult1_120_x
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $gResult2_121_z
        local.set $gResult2_121_y
        local.set $gResult2_121_x
        (f64.const 1000000)
        (f64.const 2)
        (call $_rpu_vec1_neg_f64)
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        local.set $gResult1_120_z
        local.set $gResult1_120_y
        local.set $gResult1_120_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        local.get $gResult1_120_x
        local.get $gResult1_120_y
        local.get $gResult1_120_z
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (call $_rpu_vec3_abs_f64)
        (f64.const 399.741)
        (f64.const 399.741)
        (f64.const 399.741)
        (call $_rpu_vec3_sub_vec3_f64)
        (f64.const 0)
        (call $_rpu_vec3_max_f64)
        (call $_rpu_vec3_length_f64)
        (f64.const 0.259)
        (f64.sub)
        (f64.const 0)
        (f64.const 0)
        (call $opU)
        local.set $gResult1_120_z
        local.set $gResult1_120_y
        local.set $gResult1_120_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        local.get $tp_118_y
        (f64.const 19)
        (call $_rpu_vec1_neg_f64)
        (f64.add)
        local.set $tp_118_y
        local.get $gResult1_120_x
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (call $_rpu_vec3_abs_f64)
        (f64.const 84)
        (f64.const 3)
        (f64.const 3)
        (call $_rpu_vec3_sub_vec3_f64)
        (f64.const 0)
        (call $_rpu_vec3_max_f64)
        (call $_rpu_vec3_length_f64)
        (f64.const 16)
        (f64.sub)
        (call $opS)
        local.set $gResult1_120_x
        local.get $res_116_x
        local.get $res_116_y
        local.get $res_116_z
        local.get $gResult1_120_x
        local.get $gResult1_120_y
        local.get $gResult1_120_z
        (call $opU)
        local.set $res_116_z
        local.set $res_116_y
        local.set $res_116_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (f64.const 2)
        (call $_rpu_vec1_neg_f64)
        (f64.const 2)
        (call $_rpu_vec1_neg_f64)
        (f64.const 4)
        (call $_rpu_vec1_neg_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        local.get $res_116_x
        local.get $res_116_y
        local.get $res_116_z
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (call $_rpu_vec3_length_f64)
        (f64.const 1)
        (f64.sub)
        (f64.const 1)
        (f64.const 3)
        (call $opU)
        local.set $res_116_z
        local.set $res_116_y
        local.set $res_116_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (f64.const 4)
        (f64.const 1.5)
        (call $_rpu_vec1_neg_f64)
        (f64.const 4)
        (call $_rpu_vec1_neg_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        local.get $res_116_x
        local.get $res_116_y
        local.get $res_116_z
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (call $_rpu_vec3_length_f64)
        (f64.const 0.3)
        (f64.sub)
        (f64.const 1)
        (f64.const 4)
        (call $opU)
        local.set $res_116_z
        local.set $res_116_y
        local.set $res_116_x
        (f64.const 1000000)
        (f64.const 2)
        (call $_rpu_vec1_neg_f64)
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        local.set $gResult1_120_z
        local.set $gResult1_120_y
        local.set $gResult1_120_x
        (f64.const 1000000)
        (f64.const 2)
        (call $_rpu_vec1_neg_f64)
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        local.set $gResult2_121_z
        local.set $gResult2_121_y
        local.set $gResult2_121_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        local.get $tp_118_x
        (f64.const 0.3542)
        (call $_rpu_vec1_neg_f64)
        (f64.add)
        local.set $tp_118_x
        (f64.const 1.3)
        (f64.const 1.3)
        (f64.const 1.3)
        local.set $_rpu_temp_f64
        local.get $tp_118_z
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_z
        local.set $_rpu_temp_f64
        local.get $tp_118_y
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_y
        local.set $_rpu_temp_f64
        local.get $tp_118_x
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_x
        local.get $gResult2_121_x
        local.get $gResult2_121_y
        local.get $gResult2_121_z
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (f64.const 1)
        (f64.const 0.03)
        (call $sdCylinder)
        (f64.const 3)
        (f64.const 5)
        (call $opU)
        local.set $gResult2_121_z
        local.set $gResult2_121_y
        local.set $gResult2_121_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        local.get $tp_118_x
        local.get $tp_118_y
        (f64.const 0.3542)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.13)
        (call $_rpu_vec1_neg_f64)
        (call $_rpu_vec2_add_vec2_f64)
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 1.3)
        (f64.const 1.3)
        (f64.const 1.3)
        local.set $_rpu_temp_f64
        local.get $tp_118_z
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_z
        local.set $_rpu_temp_f64
        local.get $tp_118_y
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_y
        local.set $_rpu_temp_f64
        local.get $tp_118_x
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_x
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $bumpNormal_122_z
        local.set $bumpNormal_122_y
        local.set $bumpNormal_122_x
        (i64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (i32.const 128)
        (call $malloc)
        (local.set $_rpu_temp_mem_ptr)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 120
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 112
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 104
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 96
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 88
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 80
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 72
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 64
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 56
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 48
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 40
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 32
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 24
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 16
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 8
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_i64
        local.get $_rpu_temp_mem_ptr
        i32.const 0
        i32.add
        local.get $_rpu_temp_i64
        (i64.store)
        local.get $_rpu_temp_mem_ptr
        (local.set $bumpMaterial_123)
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.get $bumpNormal_122_x
        local.get $bumpNormal_122_y
        local.get $bumpNormal_122_z
        (f64.const 0)
        (local.get $bumpMaterial_123)
        (call $material0)
        local.get $gResult2_121_x
        local.get $gResult2_121_y
        local.get $gResult2_121_z
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (f64.const 0.8)
        (f64.const 0.03)
        (call $sdCylinder)
        (local.get $bumpMaterial_123)
        (i32.const 88)
        (i32.add)
        (f64.load)
        (f64.const 50)
        (f64.div)
        (f64.sub)
        (f64.const 2)
        (f64.const 6)
        (f64.const 16.7396)
        (call $opBlend)
        local.set $gResult2_121_z
        local.set $gResult2_121_y
        local.set $gResult2_121_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        local.get $tp_118_x
        local.get $tp_118_y
        (f64.const 0.3542)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.156)
        (call $_rpu_vec1_neg_f64)
        (call $_rpu_vec2_add_vec2_f64)
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 1.3)
        (f64.const 1.3)
        (f64.const 1.3)
        local.set $_rpu_temp_f64
        local.get $tp_118_z
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_z
        local.set $_rpu_temp_f64
        local.get $tp_118_y
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_y
        local.set $_rpu_temp_f64
        local.get $tp_118_x
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_x
        local.get $gResult2_121_x
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (f64.const 0.7)
        (f64.const 0.04)
        (call $sdCylinder)
        (call $opS)
        local.set $gResult2_121_x
        local.get $gResult1_120_x
        local.get $gResult1_120_y
        local.get $gResult1_120_z
        local.get $gResult2_121_x
        local.get $gResult2_121_y
        local.get $gResult2_121_z
        (call $opU)
        local.set $gResult1_120_z
        local.set $gResult1_120_y
        local.set $gResult1_120_x
        (f64.const 1000000)
        (f64.const 2)
        (call $_rpu_vec1_neg_f64)
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        local.set $gResult2_121_z
        local.set $gResult2_121_y
        local.set $gResult2_121_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 0.3323)
        (f64.const 0.116)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.936)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0)
        (f64.const 0.3295)
        (f64.const 0.9442)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0.8838)
        (f64.const 0.3084)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.3519)
        (f64.const 0)
        (f64.const 0.4378)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.8764)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.3315)
        (f64.const 1)
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (f64.const 1)
        (call $_rpu_mat4_mul_vec4_f64)
        (local.set $_rpu_temp_f64)
        (i32.const 24)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 16)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 8)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 0)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (i32.const 0)
        (f64.load)
        (i32.const 8)
        (f64.load)
        (i32.const 16)
        (f64.load)
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 1.495)
        (f64.const 1.495)
        (f64.const 1.495)
        local.set $_rpu_temp_f64
        local.get $tp_118_z
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_z
        local.set $_rpu_temp_f64
        local.get $tp_118_y
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_y
        local.set $_rpu_temp_f64
        local.get $tp_118_x
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.get $bumpNormal_122_x
        local.get $bumpNormal_122_y
        local.get $bumpNormal_122_z
        (f64.const 0)
        (local.get $bumpMaterial_123)
        (call $material0)
        local.get $gResult2_121_x
        local.get $gResult2_121_y
        local.get $gResult2_121_z
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (call $_rpu_vec3_length_f64)
        (f64.const 0.6)
        (f64.sub)
        (local.get $bumpMaterial_123)
        (i32.const 88)
        (i32.add)
        (f64.load)
        (f64.const 50)
        (f64.div)
        (f64.sub)
        (f64.const 2)
        (f64.const 9)
        (call $opU)
        local.set $gResult2_121_z
        local.set $gResult2_121_y
        local.set $gResult2_121_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 0.84)
        (f64.const 0.5426)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.5426)
        (f64.const 0.84)
        local.get $tp_118_x
        local.get $tp_118_z
        (call $_rpu_mat2_mul_vec2_f64)
        local.set $tp_118_z
        local.set $tp_118_x
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (f64.const 0.3472)
        (call $_rpu_vec1_neg_f64)
        (f64.const 1.495)
        (call $_rpu_vec1_neg_f64)
        (f64.const 2.3441)
        (call $_rpu_vec1_neg_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 1.495)
        (f64.const 1.495)
        (f64.const 1.495)
        local.set $_rpu_temp_f64
        local.get $tp_118_z
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_z
        local.set $_rpu_temp_f64
        local.get $tp_118_y
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_y
        local.set $_rpu_temp_f64
        local.get $tp_118_x
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_x
        local.get $gResult2_121_x
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (call $_rpu_vec3_length_f64)
        (f64.const 1.28)
        (f64.sub)
        (call $opS)
        local.set $gResult2_121_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 0.84)
        (f64.const 0.0942)
        (f64.const 0.5344)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0)
        (f64.const 0)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.9848)
        (f64.const 0.1736)
        (f64.const 0)
        (f64.const 0.5426)
        (f64.const 0.1459)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.8272)
        (f64.const 0)
        (f64.const 0.2975)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.9904)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.0205)
        (f64.const 1)
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (f64.const 1)
        (call $_rpu_mat4_mul_vec4_f64)
        (local.set $_rpu_temp_f64)
        (i32.const 24)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 16)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 8)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 0)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (i32.const 0)
        (f64.load)
        (i32.const 8)
        (f64.load)
        (i32.const 16)
        (f64.load)
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 1.495)
        (f64.const 1.495)
        (f64.const 1.495)
        local.set $_rpu_temp_f64
        local.get $tp_118_z
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_z
        local.set $_rpu_temp_f64
        local.get $tp_118_y
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_y
        local.set $_rpu_temp_f64
        local.get $tp_118_x
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_x
        local.get $gResult2_121_x
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (f64.const 0.63)
        (f64.const 0.03)
        (call $sdCylinder)
        (call $opS)
        local.set $gResult2_121_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 0.84)
        (f64.const 0.5426)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.5426)
        (f64.const 0.84)
        local.get $tp_118_x
        local.get $tp_118_z
        (call $_rpu_mat2_mul_vec2_f64)
        local.set $tp_118_z
        local.set $tp_118_x
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (f64.const 0.2975)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.9717)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.1922)
        (call $_rpu_vec3_add_vec3_f64)
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 1.495)
        (f64.const 1.495)
        (f64.const 1.495)
        local.set $_rpu_temp_f64
        local.get $tp_118_z
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_z
        local.set $_rpu_temp_f64
        local.get $tp_118_y
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_y
        local.set $_rpu_temp_f64
        local.get $tp_118_x
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_x
        local.get $gResult2_121_x
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (call $_rpu_vec3_length_f64)
        (f64.const 0.55)
        (f64.sub)
        (call $opS)
        local.set $gResult2_121_x
        local.get $p_115_x
        local.get $p_115_y
        local.get $p_115_z
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 0.84)
        (f64.const 0.5426)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.5426)
        (f64.const 0.84)
        local.get $tp_118_x
        local.get $tp_118_z
        (call $_rpu_mat2_mul_vec2_f64)
        local.set $tp_118_z
        local.set $tp_118_x
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (f64.const 0.2975)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.9717)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.1922)
        (call $_rpu_vec3_add_vec3_f64)
        local.set $tp_118_z
        local.set $tp_118_y
        local.set $tp_118_x
        (f64.const 1.495)
        (f64.const 1.495)
        (f64.const 1.495)
        local.set $_rpu_temp_f64
        local.get $tp_118_z
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_z
        local.set $_rpu_temp_f64
        local.get $tp_118_y
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_y
        local.set $_rpu_temp_f64
        local.get $tp_118_x
        local.get $_rpu_temp_f64
        f64.div
        local.set $tp_118_x
        local.get $gResult2_121_x
        local.get $gResult2_121_y
        local.get $gResult2_121_z
        local.get $tp_118_x
        local.get $tp_118_y
        local.get $tp_118_z
        (call $_rpu_vec3_length_f64)
        (f64.const 0.52)
        (f64.sub)
        (f64.const 0)
        (f64.const 13)
        (call $opU)
        local.set $gResult2_121_z
        local.set $gResult2_121_y
        local.set $gResult2_121_x
        local.get $gResult1_120_x
        local.get $gResult1_120_y
        local.get $gResult1_120_z
        local.get $gResult2_121_x
        local.get $gResult2_121_y
        local.get $gResult2_121_z
        (call $opU)
        local.set $gResult1_120_z
        local.set $gResult1_120_y
        local.set $gResult1_120_x
        local.get $res_116_x
        local.get $res_116_y
        local.get $res_116_z
        local.get $gResult1_120_x
        local.get $gResult1_120_y
        local.get $gResult1_120_z
        (call $opU)
        local.set $res_116_z
        local.set $res_116_y
        local.set $res_116_x
        local.get $res_116_x
        local.get $res_116_y
        local.get $res_116_z
        (return)
    )

    ;; function 'calcNormal'
    (func $calcNormal (param $pos_124_x f64) (param $pos_124_y f64) (param $pos_124_z f64) (result f64 f64 f64)
        (local $e_125_x f64)
        (local $e_125_y f64)
        (local $_rpu_temp_f64 f64)
        (f64.const 1)
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        (f64.const 0.5773)
        (call $_rpu_vec2_mul_scalar_f64)
        (f64.const 0.0005)
        (call $_rpu_vec2_mul_scalar_f64)
        local.set $e_125_y
        local.set $e_125_x
        local.get $e_125_x
        local.get $e_125_y
        local.get $e_125_y
        local.get $pos_124_x
        local.get $pos_124_y
        local.get $pos_124_z
        local.get $e_125_x
        local.get $e_125_y
        local.get $e_125_y
        (call $_rpu_vec3_add_vec3_f64)
        (call $map)
        (local.set $_rpu_temp_f64)
        (i32.const 16)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 8)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 0)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (i32.const 0)
        (f64.load)
        (call $_rpu_vec3_mul_scalar_f64)
        local.get $e_125_y
        local.get $e_125_y
        local.get $e_125_x
        local.get $pos_124_x
        local.get $pos_124_y
        local.get $pos_124_z
        local.get $e_125_y
        local.get $e_125_y
        local.get $e_125_x
        (call $_rpu_vec3_add_vec3_f64)
        (call $map)
        (local.set $_rpu_temp_f64)
        (i32.const 16)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 8)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 0)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (i32.const 0)
        (f64.load)
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $e_125_y
        local.get $e_125_x
        local.get $e_125_y
        local.get $pos_124_x
        local.get $pos_124_y
        local.get $pos_124_z
        local.get $e_125_y
        local.get $e_125_x
        local.get $e_125_y
        (call $_rpu_vec3_add_vec3_f64)
        (call $map)
        (local.set $_rpu_temp_f64)
        (i32.const 16)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 8)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 0)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (i32.const 0)
        (f64.load)
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $e_125_x
        local.get $e_125_x
        local.get $e_125_x
        local.get $pos_124_x
        local.get $pos_124_y
        local.get $pos_124_z
        local.get $e_125_x
        local.get $e_125_x
        local.get $e_125_x
        (call $_rpu_vec3_add_vec3_f64)
        (call $map)
        (local.set $_rpu_temp_f64)
        (i32.const 16)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 8)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 0)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (i32.const 0)
        (f64.load)
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        (return)
    )

    ;; function 'castRay'
    (func $castRay (param $ro_126_x f64) (param $ro_126_y f64) (param $ro_126_z f64)(param $rd_127_x f64) (param $rd_127_y f64) (param $rd_127_z f64)(param $tmin_128 f64)(param $tmax_129 f64) (result f64 f64 f64)
        (local $t_130 f64)
        (local $m_131 f64)
        (local $id_132 f64)
        (local $precis_133 f64)
        (local $res_134_x f64)
        (local $res_134_y f64)
        (local $res_134_z f64)
        local.get $tmin_128
        local.set $t_130
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        local.set $m_131
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        local.set $id_132

        (block
            (loop
                local.get $t_130
                local.get $tmax_129
                (f64.lt)
                (i32.eqz)
                (br_if 1)
                (block
                    (f64.const 0.0005)
                    local.get $t_130
                    (f64.mul)
                    local.set $precis_133
                    local.get $ro_126_x
                    local.get $ro_126_y
                    local.get $ro_126_z
                    local.get $rd_127_x
                    local.get $rd_127_y
                    local.get $rd_127_z
                    local.get $t_130
                    (call $_rpu_vec3_mul_scalar_f64)
                    (call $_rpu_vec3_add_vec3_f64)
                    (call $map)
                    local.set $res_134_z
                    local.set $res_134_y
                    local.set $res_134_x

                    local.get $t_130
                    local.get $precis_133
                    (f64.lt)
                    local.get $t_130
                    local.get $tmax_129
                    (f64.gt)
                    (i32.or)
                    (if
                        (then
                            (br 3)
                        )
                    )
                    local.get $res_134_x
                    local.get $t_130
                    f64.add
                    local.set $t_130
                    local.get $res_134_y
                    local.set $m_131
                    local.get $res_134_z
                    local.set $id_132
                )
                (br 0)
            )
        )

        local.get $t_130
        local.get $tmax_129
        (f64.gt)
        (if
            (then
                (block
                    (f64.const 1)
                    (call $_rpu_vec1_neg_f64)
                    local.set $m_131
                    (f64.const 1)
                    (call $_rpu_vec1_neg_f64)
                    local.set $id_132
                )
            )
        )
        local.get $t_130
        local.get $m_131
        local.get $id_132
        (return)
    )

    ;; function 'ggx'
    (func $ggx (param $N_135_x f64) (param $N_135_y f64) (param $N_135_z f64)(param $V_136_x f64) (param $V_136_y f64) (param $V_136_z f64)(param $L_137_x f64) (param $L_137_y f64) (param $L_137_z f64)(param $roughness_138 f64)(param $F0_139 f64) (result f64)
        (local $H_140_x f64)
        (local $H_140_y f64)
        (local $H_140_z f64)
        (local $dotLH_141 f64)
        (local $dotNH_142 f64)
        (local $dotNL_143 f64)
        (local $dotNV_144 f64)
        (local $alpha_145 f64)
        (local $alphaSqr_146 f64)
        (local $denom_147 f64)
        (local $D_148 f64)
        (local $F_a_149 f64)
        (local $F_b_150 f64)
        (local $F_151 f64)
        (local $k_152 f64)
        (local $G_153 f64)
        local.get $V_136_x
        local.get $V_136_y
        local.get $V_136_z
        local.get $L_137_x
        local.get $L_137_y
        local.get $L_137_z
        (call $_rpu_vec3_add_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $H_140_z
        local.set $H_140_y
        local.set $H_140_x
        local.get $L_137_x
        local.get $L_137_y
        local.get $L_137_z
        local.get $H_140_x
        local.get $H_140_y
        local.get $H_140_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.const 0)
        (call $_rpu_vec1_max_f64)
        local.set $dotLH_141
        local.get $N_135_x
        local.get $N_135_y
        local.get $N_135_z
        local.get $H_140_x
        local.get $H_140_y
        local.get $H_140_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.const 0)
        (call $_rpu_vec1_max_f64)
        local.set $dotNH_142
        local.get $N_135_x
        local.get $N_135_y
        local.get $N_135_z
        local.get $L_137_x
        local.get $L_137_y
        local.get $L_137_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.const 0)
        (call $_rpu_vec1_max_f64)
        local.set $dotNL_143
        local.get $N_135_x
        local.get $N_135_y
        local.get $N_135_z
        local.get $V_136_x
        local.get $V_136_y
        local.get $V_136_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.const 0)
        (call $_rpu_vec1_max_f64)
        local.set $dotNV_144
        local.get $roughness_138
        local.get $roughness_138
        (f64.mul)
        (f64.const 0.0001)
        (f64.add)
        local.set $alpha_145
        local.get $alpha_145
        local.get $alpha_145
        (f64.mul)
        local.set $alphaSqr_146
        local.get $dotNH_142
        local.get $dotNH_142
        (f64.mul)
        local.get $alphaSqr_146
        (f64.const 1)
        (f64.sub)
        (f64.mul)
        (f64.const 1)
        (f64.add)
        local.set $denom_147
        local.get $alphaSqr_146
        local.get $denom_147
        local.get $denom_147
        (f64.mul)
        (f64.div)
        local.set $D_148
        (f64.const 1)
        local.set $F_a_149
        (f64.const 1)
        local.get $dotLH_141
        (f64.sub)
        (f64.const 5)
        (call $_rpu_vec1_pow_f64)
        local.set $F_b_150
        local.get $F_b_150
        local.get $F_a_149
        local.get $F0_139
        (call $_rpu_mix_vec1_f64)
        local.set $F_151
        local.get $alpha_145
        (f64.const 2)
        local.get $roughness_138
        (f64.mul)
        (f64.add)
        (f64.const 1)
        (f64.add)
        (f64.const 8)
        (f64.div)
        local.set $k_152
        local.get $dotNL_143
        local.get $dotNL_143
        (f64.const 1)
        local.get $k_152
        (call $_rpu_mix_vec1_f64)
        local.get $dotNV_144
        (f64.const 1)
        local.get $k_152
        (call $_rpu_mix_vec1_f64)
        (f64.mul)
        (f64.div)
        local.set $G_153
        (f64.const 0)
        (f64.const 10)
        local.get $D_148
        local.get $F_151
        (f64.mul)
        local.get $G_153
        (f64.mul)
        (f64.const 4)
        (f64.div)
        (call $_rpu_vec1_min_f64)
        (call $_rpu_vec1_max_f64)
        (return)
    )

    ;; function 'angleToDir'
    (func $angleToDir (param $n_154_x f64) (param $n_154_y f64) (param $n_154_z f64)(param $theta_155 f64)(param $phi_156 f64) (result f64 f64 f64)
        (local $sinPhi_157 f64)
        (local $cosPhi_158 f64)
        (local $w_159_x f64)
        (local $w_159_y f64)
        (local $w_159_z f64)
        (local $u_160_x f64)
        (local $u_160_y f64)
        (local $u_160_z f64)
        (local $v_161_x f64)
        (local $v_161_y f64)
        (local $v_161_z f64)
        local.get $phi_156
        (call $_rpu_vec1_sin_f64)
        local.set $sinPhi_157
        local.get $phi_156
        (call $_rpu_vec1_cos_f64)
        local.set $cosPhi_158
        local.get $n_154_x
        local.get $n_154_y
        local.get $n_154_z
        (call $_rpu_normalize_vec3_f64)
        local.set $w_159_z
        local.set $w_159_y
        local.set $w_159_x
        local.get $w_159_y
        local.get $w_159_z
        local.get $w_159_x
        local.get $w_159_x
        local.get $w_159_y
        local.get $w_159_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $u_160_z
        local.set $u_160_y
        local.set $u_160_x
        local.get $w_159_x
        local.get $w_159_y
        local.get $w_159_z
        local.get $u_160_x
        local.get $u_160_y
        local.get $u_160_z
        (call $_rpu_cross_product_f64)
        local.set $v_161_z
        local.set $v_161_y
        local.set $v_161_x
        local.get $u_160_x
        local.get $u_160_y
        local.get $u_160_z
        local.get $theta_155
        (call $_rpu_vec1_cos_f64)
        (call $_rpu_vec3_mul_scalar_f64)
        local.get $v_161_x
        local.get $v_161_y
        local.get $v_161_z
        local.get $theta_155
        (call $_rpu_vec1_sin_f64)
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $sinPhi_157
        (call $_rpu_vec3_mul_scalar_f64)
        local.get $w_159_x
        local.get $w_159_y
        local.get $w_159_z
        local.get $cosPhi_158
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        (return)
    )

    ;; function 'jitter'
    (func $jitter (param $d_162_x f64) (param $d_162_y f64) (param $d_162_z f64)(param $phi_163 f64)(param $sina_164 f64)(param $cosa_165 f64) (result f64 f64 f64)
        (local $w_166_x f64)
        (local $w_166_y f64)
        (local $w_166_z f64)
        (local $u_167_x f64)
        (local $u_167_y f64)
        (local $u_167_z f64)
        (local $v_168_x f64)
        (local $v_168_y f64)
        (local $v_168_z f64)
        local.get $d_162_x
        local.get $d_162_y
        local.get $d_162_z
        (call $_rpu_normalize_vec3_f64)
        local.set $w_166_z
        local.set $w_166_y
        local.set $w_166_x
        local.get $w_166_y
        local.get $w_166_z
        local.get $w_166_x
        local.get $w_166_x
        local.get $w_166_y
        local.get $w_166_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $u_167_z
        local.set $u_167_y
        local.set $u_167_x
        local.get $w_166_x
        local.get $w_166_y
        local.get $w_166_z
        local.get $u_167_x
        local.get $u_167_y
        local.get $u_167_z
        (call $_rpu_cross_product_f64)
        local.set $v_168_z
        local.set $v_168_y
        local.set $v_168_x
        local.get $u_167_x
        local.get $u_167_y
        local.get $u_167_z
        local.get $phi_163
        (call $_rpu_vec1_cos_f64)
        (call $_rpu_vec3_mul_scalar_f64)
        local.get $v_168_x
        local.get $v_168_y
        local.get $v_168_z
        local.get $phi_163
        (call $_rpu_vec1_sin_f64)
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $sina_164
        (call $_rpu_vec3_mul_scalar_f64)
        local.get $w_166_x
        local.get $w_166_y
        local.get $w_166_z
        local.get $cosa_165
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        (return)
    )

    ;; function 'sampleLightBRDF'
    (func $sampleLightBRDF (param $hitOrigin_169_x f64) (param $hitOrigin_169_y f64) (param $hitOrigin_169_z f64)(param $hitNormal_170_x f64) (param $hitNormal_170_y f64) (param $hitNormal_170_z f64)(param $rayDir_171_x f64) (param $rayDir_171_y f64) (param $rayDir_171_z f64)(param $material_172 i32) (result f64 f64 f64)
        (local $brdf_173_x f64)
        (local $brdf_173_y f64)
        (local $brdf_173_z f64)
        (local $s_174_x f64)
        (local $s_174_y f64)
        (local $s_174_z f64)
        (local $_rpu_temp_mem_ptr i32)
        (local $_rpu_temp_f64 f64)
        (local $light_175 i32)
        (local $l0_176_x f64)
        (local $l0_176_y f64)
        (local $l0_176_z f64)
        (local $cos_a_max_177 f64)
        (local $cosa_178 f64)
        (local $l_179_x f64)
        (local $l_179_y f64)
        (local $l_179_z f64)
        (local $lightHit_180_x f64)
        (local $lightHit_180_y f64)
        (local $lightHit_180_z f64)
        (local $roughness_181 f64)
        (local $metallic_182 f64)
        (local $omega_183 f64)
        (local $roughness_184 f64)
        (local $metallic_185 f64)
        (local $omega_186 f64)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $brdf_173_z
        local.set $brdf_173_y
        local.set $brdf_173_x
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $s_174_z
        local.set $s_174_y
        local.set $s_174_x
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (i32.const 32)
        (call $malloc)
        (local.set $_rpu_temp_mem_ptr)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 24
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 16
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 8
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 0
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.get $_rpu_temp_mem_ptr
        (local.set $light_175)
        (f64.const 3)
        (local.set $_rpu_temp_f64)
        (local.get $light_175)
        (i32.const 0)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (f64.const 180)
        (f64.const 5)
        (f64.mul)
        (f64.const 180)
        (f64.const 5)
        (f64.mul)
        (f64.const 180)
        (f64.const 5)
        (f64.mul)
        (local.set $_rpu_temp_f64)
        (local.get $light_175)
        (i32.const 24)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $light_175)
        (i32.const 16)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $light_175)
        (i32.const 8)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (f64.const 2)
        (f64.const 2)
        (f64.const 4)
        local.get $hitOrigin_169_x
        local.get $hitOrigin_169_y
        local.get $hitOrigin_169_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $l0_176_z
        local.set $l0_176_y
        local.set $l0_176_x
        (f64.const 1)
        (f64.const 0.5)
        (f64.const 0.5)
        (f64.mul)
        local.get $l0_176_x
        local.get $l0_176_y
        local.get $l0_176_z
        local.get $l0_176_x
        local.get $l0_176_y
        local.get $l0_176_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.div)
        (f64.const 0)
        (f64.const 1)
        (call $_rpu_vec1_clamp_f64_f64)
        (f64.sub)
        (call $_rpu_vec1_sqrt_f64)
        local.set $cos_a_max_177
        local.get $cos_a_max_177
        (f64.const 1)
        (call $random)
        (call $_rpu_mix_vec1_f64)
        local.set $cosa_178
        local.get $l0_176_x
        local.get $l0_176_y
        local.get $l0_176_z
        (f64.const 2)
        (f64.const 3.1415927)
        (f64.mul)
        (call $random)
        (f64.mul)
        (f64.const 1)
        local.get $cosa_178
        local.get $cosa_178
        (f64.mul)
        (f64.sub)
        (call $_rpu_vec1_sqrt_f64)
        local.get $cosa_178
        (call $jitter)
        local.set $l_179_z
        local.set $l_179_y
        local.set $l_179_x
        local.get $hitOrigin_169_x
        local.get $hitOrigin_169_y
        local.get $hitOrigin_169_z
        local.get $l_179_x
        local.get $l_179_y
        local.get $l_179_z
        (f64.const 0.001)
        (f64.const 100)
        (call $castRay)
        local.set $lightHit_180_z
        local.set $lightHit_180_y
        local.set $lightHit_180_x

        local.get $lightHit_180_z
        (local.get $light_175)
        (i32.const 0)
        (i32.add)
        (f64.load)
        (f64.eq)
        (if
            (then
                (block
                    (f64.const 1)
                    (local.get $material_172)
                    (i32.const 72)
                    (i32.add)
                    (f64.load)
                    (local.get $material_172)
                    (i32.const 72)
                    (i32.add)
                    (f64.load)
                    (f64.mul)
                    (f64.sub)
                    local.set $roughness_181
                    (local.get $material_172)
                    (i32.const 64)
                    (i32.add)
                    (f64.load)
                    local.set $metallic_182
                    (f64.const 2)
                    (f64.const 3.1415927)
                    (f64.mul)
                    (f64.const 1)
                    local.get $cos_a_max_177
                    (f64.sub)
                    (f64.mul)
                    local.set $omega_183
                    (local.get $light_175)
                    (i32.const 8)
                    (i32.add)
                    (f64.load)
                    (local.get $light_175)
                    (i32.const 16)
                    (i32.add)
                    (f64.load)
                    (local.get $light_175)
                    (i32.const 24)
                    (i32.add)
                    (f64.load)
                    local.get $hitNormal_170_x
                    local.get $hitNormal_170_y
                    local.get $hitNormal_170_z
                    local.get $rayDir_171_x
                    local.get $rayDir_171_y
                    local.get $rayDir_171_z
                    local.get $l_179_x
                    local.get $l_179_y
                    local.get $l_179_z
                    local.get $roughness_181
                    local.get $metallic_182
                    (call $ggx)
                    (f64.const 0)
                    (f64.const 1)
                    (call $_rpu_vec1_clamp_f64_f64)
                    (call $_rpu_vec3_mul_scalar_f64)
                    local.get $omega_183
                    (call $_rpu_vec3_mul_scalar_f64)
                    (f64.const 3.1415927)
                    (call $_rpu_vec3_div_scalar_f64)
                    local.set $_rpu_temp_f64
                    local.get $brdf_173_z
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $brdf_173_z
                    local.set $_rpu_temp_f64
                    local.get $brdf_173_y
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $brdf_173_y
                    local.set $_rpu_temp_f64
                    local.get $brdf_173_x
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $brdf_173_x
                )
            )
        )
        (f64.const 4)
        (local.set $_rpu_temp_f64)
        (local.get $light_175)
        (i32.const 0)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (f64.const 4)
        (call $_rpu_vec1_neg_f64)
        (f64.const 1.5)
        (f64.const 4)
        local.get $hitOrigin_169_x
        local.get $hitOrigin_169_y
        local.get $hitOrigin_169_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $l0_176_z
        local.set $l0_176_y
        local.set $l0_176_x
        (f64.const 1)
        (f64.const 0.5)
        (f64.const 0.5)
        (f64.mul)
        local.get $l0_176_x
        local.get $l0_176_y
        local.get $l0_176_z
        local.get $l0_176_x
        local.get $l0_176_y
        local.get $l0_176_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.div)
        (f64.const 0)
        (f64.const 1)
        (call $_rpu_vec1_clamp_f64_f64)
        (f64.sub)
        (call $_rpu_vec1_sqrt_f64)
        local.set $cos_a_max_177
        local.get $cos_a_max_177
        (f64.const 1)
        (call $random)
        (call $_rpu_mix_vec1_f64)
        local.set $cosa_178
        local.get $l0_176_x
        local.get $l0_176_y
        local.get $l0_176_z
        (f64.const 2)
        (f64.const 3.1415927)
        (f64.mul)
        (call $random)
        (f64.mul)
        (f64.const 1)
        local.get $cosa_178
        local.get $cosa_178
        (f64.mul)
        (f64.sub)
        (call $_rpu_vec1_sqrt_f64)
        local.get $cosa_178
        (call $jitter)
        local.set $l_179_z
        local.set $l_179_y
        local.set $l_179_x
        local.get $hitOrigin_169_x
        local.get $hitOrigin_169_y
        local.get $hitOrigin_169_z
        local.get $l_179_x
        local.get $l_179_y
        local.get $l_179_z
        (f64.const 0.001)
        (f64.const 100)
        (call $castRay)
        local.set $lightHit_180_z
        local.set $lightHit_180_y
        local.set $lightHit_180_x

        local.get $lightHit_180_z
        (local.get $light_175)
        (i32.const 0)
        (i32.add)
        (f64.load)
        (f64.eq)
        (if
            (then
                (block
                    (f64.const 1)
                    (local.get $material_172)
                    (i32.const 72)
                    (i32.add)
                    (f64.load)
                    (local.get $material_172)
                    (i32.const 72)
                    (i32.add)
                    (f64.load)
                    (f64.mul)
                    (f64.sub)
                    local.set $roughness_184
                    (local.get $material_172)
                    (i32.const 64)
                    (i32.add)
                    (f64.load)
                    local.set $metallic_185
                    (f64.const 2)
                    (f64.const 3.1415927)
                    (f64.mul)
                    (f64.const 1)
                    local.get $cos_a_max_177
                    (f64.sub)
                    (f64.mul)
                    local.set $omega_186
                    (local.get $light_175)
                    (i32.const 8)
                    (i32.add)
                    (f64.load)
                    (local.get $light_175)
                    (i32.const 16)
                    (i32.add)
                    (f64.load)
                    (local.get $light_175)
                    (i32.const 24)
                    (i32.add)
                    (f64.load)
                    local.get $hitNormal_170_x
                    local.get $hitNormal_170_y
                    local.get $hitNormal_170_z
                    local.get $rayDir_171_x
                    local.get $rayDir_171_y
                    local.get $rayDir_171_z
                    local.get $l_179_x
                    local.get $l_179_y
                    local.get $l_179_z
                    local.get $roughness_184
                    local.get $metallic_185
                    (call $ggx)
                    (f64.const 0)
                    (f64.const 1)
                    (call $_rpu_vec1_clamp_f64_f64)
                    (call $_rpu_vec3_mul_scalar_f64)
                    local.get $omega_186
                    (call $_rpu_vec3_mul_scalar_f64)
                    (f64.const 3.1415927)
                    (call $_rpu_vec3_div_scalar_f64)
                    local.set $_rpu_temp_f64
                    local.get $brdf_173_z
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $brdf_173_z
                    local.set $_rpu_temp_f64
                    local.get $brdf_173_y
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $brdf_173_y
                    local.set $_rpu_temp_f64
                    local.get $brdf_173_x
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $brdf_173_x
                )
            )
        )
        local.get $brdf_173_x
        local.get $brdf_173_y
        local.get $brdf_173_z
        (return)
    )

    ;; function 'sampleLightE'
    (func $sampleLightE (param $hitOrigin_187_x f64) (param $hitOrigin_187_y f64) (param $hitOrigin_187_z f64)(param $hitNormal_188_x f64) (param $hitNormal_188_y f64) (param $hitNormal_188_z f64)(param $rayDir_189_x f64) (param $rayDir_189_y f64) (param $rayDir_189_z f64)(param $material_190 i32) (result f64 f64 f64)
        (local $e_191_x f64)
        (local $e_191_y f64)
        (local $e_191_z f64)
        (local $s_192_x f64)
        (local $s_192_y f64)
        (local $s_192_z f64)
        (local $_rpu_temp_mem_ptr i32)
        (local $_rpu_temp_f64 f64)
        (local $light_193 i32)
        (local $l0_194_x f64)
        (local $l0_194_y f64)
        (local $l0_194_z f64)
        (local $cos_a_max_195 f64)
        (local $cosa_196 f64)
        (local $l_197_x f64)
        (local $l_197_y f64)
        (local $l_197_z f64)
        (local $lightHit_198_x f64)
        (local $lightHit_198_y f64)
        (local $lightHit_198_z f64)
        (local $omega_199 f64)
        (local $n_200_x f64)
        (local $n_200_y f64)
        (local $n_200_z f64)
        (local $omega_201 f64)
        (local $n_202_x f64)
        (local $n_202_y f64)
        (local $n_202_z f64)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $e_191_z
        local.set $e_191_y
        local.set $e_191_x
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $s_192_z
        local.set $s_192_y
        local.set $s_192_x
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (i32.const 32)
        (call $malloc)
        (local.set $_rpu_temp_mem_ptr)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 24
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 16
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 8
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 0
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.get $_rpu_temp_mem_ptr
        (local.set $light_193)
        (f64.const 3)
        (local.set $_rpu_temp_f64)
        (local.get $light_193)
        (i32.const 0)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (f64.const 180)
        (f64.const 5)
        (f64.mul)
        (f64.const 180)
        (f64.const 5)
        (f64.mul)
        (f64.const 180)
        (f64.const 5)
        (f64.mul)
        (local.set $_rpu_temp_f64)
        (local.get $light_193)
        (i32.const 24)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $light_193)
        (i32.const 16)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $light_193)
        (i32.const 8)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (f64.const 0.5)
        (f64.const 0.5)
        (f64.const 8)
        local.get $hitOrigin_187_x
        local.get $hitOrigin_187_y
        local.get $hitOrigin_187_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $l0_194_z
        local.set $l0_194_y
        local.set $l0_194_x
        (f64.const 1)
        (f64.const 0.5)
        (f64.const 0.5)
        (f64.mul)
        local.get $l0_194_x
        local.get $l0_194_y
        local.get $l0_194_z
        local.get $l0_194_x
        local.get $l0_194_y
        local.get $l0_194_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.div)
        (f64.const 0)
        (f64.const 1)
        (call $_rpu_vec1_clamp_f64_f64)
        (f64.sub)
        (call $_rpu_vec1_sqrt_f64)
        local.set $cos_a_max_195
        local.get $cos_a_max_195
        (f64.const 1)
        (call $random)
        (call $_rpu_mix_vec1_f64)
        local.set $cosa_196
        local.get $l0_194_x
        local.get $l0_194_y
        local.get $l0_194_z
        (f64.const 2)
        (f64.const 3.1415927)
        (f64.mul)
        (call $random)
        (f64.mul)
        (f64.const 1)
        local.get $cosa_196
        local.get $cosa_196
        (f64.mul)
        (f64.sub)
        (call $_rpu_vec1_sqrt_f64)
        local.get $cosa_196
        (call $jitter)
        local.set $l_197_z
        local.set $l_197_y
        local.set $l_197_x
        local.get $hitOrigin_187_x
        local.get $hitOrigin_187_y
        local.get $hitOrigin_187_z
        local.get $l_197_x
        local.get $l_197_y
        local.get $l_197_z
        (f64.const 0.001)
        (f64.const 100)
        (call $castRay)
        local.set $lightHit_198_z
        local.set $lightHit_198_y
        local.set $lightHit_198_x

        local.get $lightHit_198_z
        (local.get $light_193)
        (i32.const 0)
        (i32.add)
        (f64.load)
        (f64.eq)
        (if
            (then
                (block
                    (f64.const 2)
                    (f64.const 3.1415927)
                    (f64.mul)
                    (f64.const 1)
                    local.get $cos_a_max_195
                    (f64.sub)
                    (f64.mul)
                    local.set $omega_199
                    local.get $hitOrigin_187_x
                    local.get $hitOrigin_187_y
                    local.get $hitOrigin_187_z
                    (f64.const 0.5)
                    (f64.const 0.5)
                    (f64.const 8)
                    (call $_rpu_vec3_sub_vec3_f64)
                    (call $_rpu_normalize_vec3_f64)
                    local.set $n_200_z
                    local.set $n_200_y
                    local.set $n_200_x
                    (local.get $light_193)
                    (i32.const 8)
                    (i32.add)
                    (f64.load)
                    (local.get $light_193)
                    (i32.const 16)
                    (i32.add)
                    (f64.load)
                    (local.get $light_193)
                    (i32.const 24)
                    (i32.add)
                    (f64.load)
                    local.get $l_197_x
                    local.get $l_197_y
                    local.get $l_197_z
                    local.get $n_200_x
                    local.get $n_200_y
                    local.get $n_200_z
                    (call $_rpu_dot_product_vec3_f64)
                    (f64.const 0)
                    (f64.const 1)
                    (call $_rpu_vec1_clamp_f64_f64)
                    (call $_rpu_vec3_mul_scalar_f64)
                    local.get $omega_199
                    (call $_rpu_vec3_mul_scalar_f64)
                    (f64.const 3.1415927)
                    (call $_rpu_vec3_div_scalar_f64)
                    local.set $_rpu_temp_f64
                    local.get $e_191_z
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $e_191_z
                    local.set $_rpu_temp_f64
                    local.get $e_191_y
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $e_191_y
                    local.set $_rpu_temp_f64
                    local.get $e_191_x
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $e_191_x
                )
            )
        )
        (f64.const 4)
        (local.set $_rpu_temp_f64)
        (local.get $light_193)
        (i32.const 0)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (f64.const 4)
        (call $_rpu_vec1_neg_f64)
        (f64.const 1.5)
        (f64.const 4)
        local.get $hitOrigin_187_x
        local.get $hitOrigin_187_y
        local.get $hitOrigin_187_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $l0_194_z
        local.set $l0_194_y
        local.set $l0_194_x
        (f64.const 1)
        (f64.const 0.5)
        (f64.const 0.5)
        (f64.mul)
        local.get $l0_194_x
        local.get $l0_194_y
        local.get $l0_194_z
        local.get $l0_194_x
        local.get $l0_194_y
        local.get $l0_194_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.div)
        (f64.const 0)
        (f64.const 1)
        (call $_rpu_vec1_clamp_f64_f64)
        (f64.sub)
        (call $_rpu_vec1_sqrt_f64)
        local.set $cos_a_max_195
        local.get $cos_a_max_195
        (f64.const 1)
        (call $random)
        (call $_rpu_mix_vec1_f64)
        local.set $cosa_196
        local.get $l0_194_x
        local.get $l0_194_y
        local.get $l0_194_z
        (f64.const 2)
        (f64.const 3.1415927)
        (f64.mul)
        (call $random)
        (f64.mul)
        (f64.const 1)
        local.get $cosa_196
        local.get $cosa_196
        (f64.mul)
        (f64.sub)
        (call $_rpu_vec1_sqrt_f64)
        local.get $cosa_196
        (call $jitter)
        local.set $l_197_z
        local.set $l_197_y
        local.set $l_197_x
        local.get $hitOrigin_187_x
        local.get $hitOrigin_187_y
        local.get $hitOrigin_187_z
        local.get $l_197_x
        local.get $l_197_y
        local.get $l_197_z
        (f64.const 0.001)
        (f64.const 100)
        (call $castRay)
        local.set $lightHit_198_z
        local.set $lightHit_198_y
        local.set $lightHit_198_x

        local.get $lightHit_198_z
        (local.get $light_193)
        (i32.const 0)
        (i32.add)
        (f64.load)
        (f64.eq)
        (if
            (then
                (block
                    (f64.const 2)
                    (f64.const 3.1415927)
                    (f64.mul)
                    (f64.const 1)
                    local.get $cos_a_max_195
                    (f64.sub)
                    (f64.mul)
                    local.set $omega_201
                    local.get $hitOrigin_187_x
                    local.get $hitOrigin_187_y
                    local.get $hitOrigin_187_z
                    (f64.const 4)
                    (call $_rpu_vec1_neg_f64)
                    (f64.const 1.5)
                    (f64.const 4)
                    (call $_rpu_vec3_sub_vec3_f64)
                    (call $_rpu_normalize_vec3_f64)
                    local.set $n_202_z
                    local.set $n_202_y
                    local.set $n_202_x
                    (local.get $light_193)
                    (i32.const 8)
                    (i32.add)
                    (f64.load)
                    (local.get $light_193)
                    (i32.const 16)
                    (i32.add)
                    (f64.load)
                    (local.get $light_193)
                    (i32.const 24)
                    (i32.add)
                    (f64.load)
                    local.get $l_197_x
                    local.get $l_197_y
                    local.get $l_197_z
                    local.get $n_202_x
                    local.get $n_202_y
                    local.get $n_202_z
                    (call $_rpu_dot_product_vec3_f64)
                    (f64.const 0)
                    (f64.const 1)
                    (call $_rpu_vec1_clamp_f64_f64)
                    (call $_rpu_vec3_mul_scalar_f64)
                    local.get $omega_201
                    (call $_rpu_vec3_mul_scalar_f64)
                    (f64.const 3.1415927)
                    (call $_rpu_vec3_div_scalar_f64)
                    local.set $_rpu_temp_f64
                    local.get $e_191_z
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $e_191_z
                    local.set $_rpu_temp_f64
                    local.get $e_191_y
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $e_191_y
                    local.set $_rpu_temp_f64
                    local.get $e_191_x
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $e_191_x
                )
            )
        )
        local.get $e_191_x
        local.get $e_191_y
        local.get $e_191_z
        (return)
    )

    ;; function 'getColor'
    (func $getColor (param $ray_203 i32) (result f64 f64 f64 f64)
        (local $tcol_204_x f64)
        (local $tcol_204_y f64)
        (local $tcol_204_z f64)
        (local $tcol_204_w f64)
        (local $fcol_205_x f64)
        (local $fcol_205_y f64)
        (local $fcol_205_z f64)
        (local $fcol_205_w f64)
        (local $depth_206 i64)
        (local $_rpu_temp_mem_ptr i32)
        (local $_rpu_temp_f64 f64)
        (local $_rpu_temp_i64 i64)
        (local $material_207 i32)
        (local $normal_208_x f64)
        (local $normal_208_y f64)
        (local $normal_208_z f64)
        (local $hit_209_x f64)
        (local $hit_209_y f64)
        (local $hit_209_z f64)
        (local $hitOrigin_210_x f64)
        (local $hitOrigin_210_y f64)
        (local $hitOrigin_210_z f64)
        (local $E_211 f64)
        (local $roughness_212 f64)
        (local $alpha_213 f64)
        (local $metallic_214 f64)
        (local $reflectance_215 f64)
        (local $specular_216 f64)
        (local $diffuse_217 f64)
        (local $color_218_x f64)
        (local $color_218_y f64)
        (local $color_218_z f64)
        (local $color_218_w f64)
        (local $brdf_219_x f64)
        (local $brdf_219_y f64)
        (local $brdf_219_z f64)
        (local $brdf_220_x f64)
        (local $brdf_220_y f64)
        (local $brdf_220_z f64)
        (local $rand_221_x f64)
        (local $rand_221_y f64)
        (local $xsi_1_222 f64)
        (local $xsi_2_223 f64)
        (local $phi_224 f64)
        (local $theta_225 f64)
        (local $direction_226_x f64)
        (local $direction_226_y f64)
        (local $direction_226_z f64)
        (local $r2_227 f64)
        (local $d_228_x f64)
        (local $d_228_y f64)
        (local $d_228_z f64)
        (local $e_229_x f64)
        (local $e_229_y f64)
        (local $e_229_z f64)
        (local $E_230 f64)
        (local $backColor_231_x f64)
        (local $backColor_231_y f64)
        (local $backColor_231_z f64)
        (local $backColor_231_w f64)
        (f64.const 1)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $tcol_204_w
        local.set $tcol_204_z
        local.set $tcol_204_y
        local.set $tcol_204_x
        (f64.const 1)
        (f64.const 1)
        (f64.const 1)
        (f64.const 1)
        local.set $fcol_205_w
        local.set $fcol_205_z
        local.set $fcol_205_y
        local.set $fcol_205_x

        (i64.const 0)
        local.set $depth_206
        (block
            (loop
                local.get $depth_206
                (i64.const 2)
                (i64.lt_s)
                (i32.eqz)
                (br_if 1)
                (block
                    (i64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (i32.const 128)
                    (call $malloc)
                    (local.set $_rpu_temp_mem_ptr)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 120
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 112
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 104
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 96
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 88
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 80
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 72
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 64
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 56
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 48
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 40
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 32
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 24
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 16
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_f64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 8
                    i32.add
                    local.get $_rpu_temp_f64
                    (f64.store)
                    local.set $_rpu_temp_i64
                    local.get $_rpu_temp_mem_ptr
                    i32.const 0
                    i32.add
                    local.get $_rpu_temp_i64
                    (i64.store)
                    local.get $_rpu_temp_mem_ptr
                    (local.set $material_207)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    local.set $normal_208_z
                    local.set $normal_208_y
                    local.set $normal_208_x
                    (local.get $ray_203)
                    (i32.const 0)
                    (i32.add)
                    (f64.load)
                    (local.get $ray_203)
                    (i32.const 8)
                    (i32.add)
                    (f64.load)
                    (local.get $ray_203)
                    (i32.const 16)
                    (i32.add)
                    (f64.load)
                    (local.get $ray_203)
                    (i32.const 24)
                    (i32.add)
                    (f64.load)
                    (local.get $ray_203)
                    (i32.const 32)
                    (i32.add)
                    (f64.load)
                    (local.get $ray_203)
                    (i32.const 40)
                    (i32.add)
                    (f64.load)
                    (f64.const 0.001)
                    (f64.const 100)
                    (call $castRay)
                    local.set $hit_209_z
                    local.set $hit_209_y
                    local.set $hit_209_x

                    local.get $hit_209_y
                    (f64.const 0)
                    (f64.ge)
                    (if
                        (then
                            (block
                                (local.get $ray_203)
                                (i32.const 0)
                                (i32.add)
                                (f64.load)
                                (local.get $ray_203)
                                (i32.const 8)
                                (i32.add)
                                (f64.load)
                                (local.get $ray_203)
                                (i32.const 16)
                                (i32.add)
                                (f64.load)
                                (local.get $ray_203)
                                (i32.const 24)
                                (i32.add)
                                (f64.load)
                                (local.get $ray_203)
                                (i32.const 32)
                                (i32.add)
                                (f64.load)
                                (local.get $ray_203)
                                (i32.const 40)
                                (i32.add)
                                (f64.load)
                                local.get $hit_209_x
                                (call $_rpu_vec3_mul_scalar_f64)
                                (call $_rpu_vec3_add_vec3_f64)
                                local.set $hitOrigin_210_z
                                local.set $hitOrigin_210_y
                                local.set $hitOrigin_210_x
                                local.get $hitOrigin_210_x
                                local.get $hitOrigin_210_y
                                local.get $hitOrigin_210_z
                                (call $calcNormal)
                                local.set $normal_208_z
                                local.set $normal_208_y
                                local.set $normal_208_x
                                (f64.const 0)
                                (f64.const 0)
                                (f64.const 0)
                                (local.set $_rpu_temp_f64)
                                (local.get $material_207)
                                (i32.const 112)
                                (i32.add)
                                (local.get $_rpu_temp_f64)
                                (f64.store)
                                (local.set $_rpu_temp_f64)
                                (local.get $material_207)
                                (i32.const 104)
                                (i32.add)
                                (local.get $_rpu_temp_f64)
                                (f64.store)
                                (local.set $_rpu_temp_f64)
                                (local.get $material_207)
                                (i32.const 96)
                                (i32.add)
                                (local.get $_rpu_temp_f64)
                                (f64.store)
                                (f64.const 1)
                                (f64.const 1)
                                (f64.const 1)
                                (local.set $_rpu_temp_f64)
                                (local.get $material_207)
                                (i32.const 48)
                                (i32.add)
                                (local.get $_rpu_temp_f64)
                                (f64.store)
                                (local.set $_rpu_temp_f64)
                                (local.get $material_207)
                                (i32.const 40)
                                (i32.add)
                                (local.get $_rpu_temp_f64)
                                (f64.store)
                                (local.set $_rpu_temp_f64)
                                (local.get $material_207)
                                (i32.const 32)
                                (i32.add)
                                (local.get $_rpu_temp_f64)
                                (f64.store)
                                (f64.const 0)
                                (local.set $_rpu_temp_f64)
                                (local.get $material_207)
                                (i32.const 56)
                                (i32.add)
                                (local.get $_rpu_temp_f64)
                                (f64.store)

                                local.get $hit_209_y
                                (f64.const 0)
                                (f64.eq)
                                (if
                                    (then
                                        (block
                                            (i64.const 0)
                                            (local.set $_rpu_temp_i64)
                                            (local.get $material_207)
                                            (i32.const 0)
                                            (i32.add)
                                            (local.get $_rpu_temp_i64)
                                            (i64.store)
                                            (f64.const 0.996)
                                            (f64.const 0.929)
                                            (f64.const 0.929)
                                            (local.set $_rpu_temp_f64)
                                            (local.get $material_207)
                                            (i32.const 24)
                                            (i32.add)
                                            (local.get $_rpu_temp_f64)
                                            (f64.store)
                                            (local.set $_rpu_temp_f64)
                                            (local.get $material_207)
                                            (i32.const 16)
                                            (i32.add)
                                            (local.get $_rpu_temp_f64)
                                            (f64.store)
                                            (local.set $_rpu_temp_f64)
                                            (local.get $material_207)
                                            (i32.const 8)
                                            (i32.add)
                                            (local.get $_rpu_temp_f64)
                                            (f64.store)
                                            (f64.const 0)
                                            (local.set $_rpu_temp_f64)
                                            (local.get $material_207)
                                            (i32.const 64)
                                            (i32.add)
                                            (local.get $_rpu_temp_f64)
                                            (f64.store)
                                            (f64.const 0)
                                            (local.set $_rpu_temp_f64)
                                            (local.get $material_207)
                                            (i32.const 72)
                                            (i32.add)
                                            (local.get $_rpu_temp_f64)
                                            (f64.store)
                                            (f64.const 1)
                                            (local.set $_rpu_temp_f64)
                                            (local.get $material_207)
                                            (i32.const 80)
                                            (i32.add)
                                            (local.get $_rpu_temp_f64)
                                            (f64.store)
                                        )
                                    )
                                    (else

                                        local.get $hit_209_y
                                        (f64.const 1)
                                        (f64.eq)
                                        (if
                                            (then
                                                (block
                                                    (i64.const 2)
                                                    (local.set $_rpu_temp_i64)
                                                    (local.get $material_207)
                                                    (i32.const 0)
                                                    (i32.add)
                                                    (local.get $_rpu_temp_i64)
                                                    (i64.store)
                                                    (f64.const 200)
                                                    (f64.const 200)
                                                    (f64.const 200)
                                                    (local.set $_rpu_temp_f64)
                                                    (local.get $material_207)
                                                    (i32.const 112)
                                                    (i32.add)
                                                    (local.get $_rpu_temp_f64)
                                                    (f64.store)
                                                    (local.set $_rpu_temp_f64)
                                                    (local.get $material_207)
                                                    (i32.const 104)
                                                    (i32.add)
                                                    (local.get $_rpu_temp_f64)
                                                    (f64.store)
                                                    (local.set $_rpu_temp_f64)
                                                    (local.get $material_207)
                                                    (i32.const 96)
                                                    (i32.add)
                                                    (local.get $_rpu_temp_f64)
                                                    (f64.store)
                                                )
                                            )
                                            (else

                                                local.get $hit_209_y
                                                (f64.const 2)
                                                (f64.eq)
                                                (if
                                                    (then
                                                        (block
                                                            local.get $hitOrigin_210_x
                                                            local.get $hitOrigin_210_y
                                                            local.get $hitOrigin_210_z
                                                            local.get $normal_208_x
                                                            local.get $normal_208_y
                                                            local.get $normal_208_z
                                                            (f64.const 0)
                                                            (local.get $material_207)
                                                            (call $material0)
                                                        )
                                                    )
                                                    (else

                                                        local.get $hit_209_y
                                                        (f64.const 3)
                                                        (f64.eq)
                                                        (if
                                                            (then
                                                                (block
                                                                    (i64.const 0)
                                                                    (local.set $_rpu_temp_i64)
                                                                    (local.get $material_207)
                                                                    (i32.const 0)
                                                                    (i32.add)
                                                                    (local.get $_rpu_temp_i64)
                                                                    (i64.store)
                                                                    (f64.const 1)
                                                                    (f64.const 1)
                                                                    (f64.const 1)
                                                                    (local.set $_rpu_temp_f64)
                                                                    (local.get $material_207)
                                                                    (i32.const 24)
                                                                    (i32.add)
                                                                    (local.get $_rpu_temp_f64)
                                                                    (f64.store)
                                                                    (local.set $_rpu_temp_f64)
                                                                    (local.get $material_207)
                                                                    (i32.const 16)
                                                                    (i32.add)
                                                                    (local.get $_rpu_temp_f64)
                                                                    (f64.store)
                                                                    (local.set $_rpu_temp_f64)
                                                                    (local.get $material_207)
                                                                    (i32.const 8)
                                                                    (i32.add)
                                                                    (local.get $_rpu_temp_f64)
                                                                    (f64.store)
                                                                    (f64.const 0)
                                                                    (local.set $_rpu_temp_f64)
                                                                    (local.get $material_207)
                                                                    (i32.const 64)
                                                                    (i32.add)
                                                                    (local.get $_rpu_temp_f64)
                                                                    (f64.store)
                                                                    (f64.const 0.4)
                                                                    (local.set $_rpu_temp_f64)
                                                                    (local.get $material_207)
                                                                    (i32.const 72)
                                                                    (i32.add)
                                                                    (local.get $_rpu_temp_f64)
                                                                    (f64.store)
                                                                    (f64.const 1)
                                                                    (local.set $_rpu_temp_f64)
                                                                    (local.get $material_207)
                                                                    (i32.const 80)
                                                                    (i32.add)
                                                                    (local.get $_rpu_temp_f64)
                                                                    (f64.store)
                                                                )
                                                            )
                                                        )
                                                    )
                                                )
                                            )
                                        )
                                    )
                                )

                                (local.get $material_207)
                                (i32.const 0)
                                (i32.add)
                                (i64.load)
                                (i64.const 0)
                                (i64.eq)
                                (if
                                    (then
                                        (block
                                            (f64.const 1)
                                            local.set $E_211
                                            (f64.const 1)
                                            (local.get $material_207)
                                            (i32.const 72)
                                            (i32.add)
                                            (f64.load)
                                            (local.get $material_207)
                                            (i32.const 72)
                                            (i32.add)
                                            (f64.load)
                                            (f64.mul)
                                            (f64.sub)
                                            local.set $roughness_212
                                            local.get $roughness_212
                                            local.get $roughness_212
                                            (f64.mul)
                                            local.set $alpha_213
                                            (local.get $material_207)
                                            (i32.const 64)
                                            (i32.add)
                                            (f64.load)
                                            local.set $metallic_214
                                            (local.get $material_207)
                                            (i32.const 80)
                                            (i32.add)
                                            (f64.load)
                                            local.set $reflectance_215
                                            (local.get $material_207)
                                            (i32.const 56)
                                            (i32.add)
                                            (f64.load)
                                            local.set $specular_216
                                            (f64.const 1)
                                            local.get $specular_216
                                            (f64.sub)
                                            local.set $diffuse_217
                                            (local.get $material_207)
                                            (i32.const 8)
                                            (i32.add)
                                            (f64.load)
                                            (local.get $material_207)
                                            (i32.const 16)
                                            (i32.add)
                                            (f64.load)
                                            (local.get $material_207)
                                            (i32.const 24)
                                            (i32.add)
                                            (f64.load)
                                            local.get $diffuse_217
                                            (call $_rpu_vec3_mul_scalar_f64)
                                            (local.get $material_207)
                                            (i32.const 32)
                                            (i32.add)
                                            (f64.load)
                                            (local.get $material_207)
                                            (i32.const 40)
                                            (i32.add)
                                            (f64.load)
                                            (local.get $material_207)
                                            (i32.const 48)
                                            (i32.add)
                                            (f64.load)
                                            local.get $specular_216
                                            (call $_rpu_vec3_mul_scalar_f64)
                                            (call $_rpu_vec3_add_vec3_f64)
                                            (f64.const 1)
                                            local.set $color_218_w
                                            local.set $color_218_z
                                            local.set $color_218_y
                                            local.set $color_218_x
                                            (f64.const 0)
                                            (f64.const 0)
                                            (f64.const 0)
                                            local.set $brdf_219_z
                                            local.set $brdf_219_y
                                            local.set $brdf_219_x

                                            (call $random)
                                            local.get $reflectance_215
                                            (f64.lt)
                                            (if
                                                (then
                                                    (block
                                                        local.get $hitOrigin_210_x
                                                        local.get $hitOrigin_210_y
                                                        local.get $hitOrigin_210_z
                                                        local.get $normal_208_x
                                                        local.get $normal_208_y
                                                        local.get $normal_208_z
                                                        (local.get $ray_203)
                                                        (i32.const 24)
                                                        (i32.add)
                                                        (f64.load)
                                                        (local.get $ray_203)
                                                        (i32.const 32)
                                                        (i32.add)
                                                        (f64.load)
                                                        (local.get $ray_203)
                                                        (i32.const 40)
                                                        (i32.add)
                                                        (f64.load)
                                                        (local.get $material_207)
                                                        (call $sampleLightBRDF)
                                                        local.set $brdf_220_z
                                                        local.set $brdf_220_y
                                                        local.set $brdf_220_x
                                                        (call $rand2)
                                                        local.set $rand_221_y
                                                        local.set $rand_221_x
                                                        local.get $rand_221_x
                                                        local.set $xsi_1_222
                                                        local.get $rand_221_y
                                                        local.set $xsi_2_223
                                                        local.get $alpha_213
                                                        local.get $xsi_1_222
                                                        (call $_rpu_vec1_sqrt_f64)
                                                        (f64.mul)
                                                        (f64.const 1)
                                                        local.get $xsi_1_222
                                                        (f64.sub)
                                                        (call $_rpu_vec1_sqrt_f64)
                                                        (f64.div)
                                                        (call $_rpu_vec1_atan_f64)
                                                        local.set $phi_224
                                                        (f64.const 2)
                                                        (f64.const 3.1415927)
                                                        (f64.mul)
                                                        local.get $xsi_2_223
                                                        (f64.mul)
                                                        local.set $theta_225
                                                        local.get $normal_208_x
                                                        local.get $normal_208_y
                                                        local.get $normal_208_z
                                                        local.get $theta_225
                                                        local.get $phi_224
                                                        (call $angleToDir)
                                                        local.set $direction_226_z
                                                        local.set $direction_226_y
                                                        local.set $direction_226_x
                                                        local.get $hitOrigin_210_x
                                                        local.get $hitOrigin_210_y
                                                        local.get $hitOrigin_210_z
                                                        local.get $direction_226_x
                                                        local.get $direction_226_y
                                                        local.get $direction_226_z
                                                        (i32.const 48)
                                                        (call $malloc)
                                                        (local.set $_rpu_temp_mem_ptr)
                                                        local.set $_rpu_temp_f64
                                                        local.get $_rpu_temp_mem_ptr
                                                        i32.const 40
                                                        i32.add
                                                        local.get $_rpu_temp_f64
                                                        (f64.store)
                                                        local.set $_rpu_temp_f64
                                                        local.get $_rpu_temp_mem_ptr
                                                        i32.const 32
                                                        i32.add
                                                        local.get $_rpu_temp_f64
                                                        (f64.store)
                                                        local.set $_rpu_temp_f64
                                                        local.get $_rpu_temp_mem_ptr
                                                        i32.const 24
                                                        i32.add
                                                        local.get $_rpu_temp_f64
                                                        (f64.store)
                                                        local.set $_rpu_temp_f64
                                                        local.get $_rpu_temp_mem_ptr
                                                        i32.const 16
                                                        i32.add
                                                        local.get $_rpu_temp_f64
                                                        (f64.store)
                                                        local.set $_rpu_temp_f64
                                                        local.get $_rpu_temp_mem_ptr
                                                        i32.const 8
                                                        i32.add
                                                        local.get $_rpu_temp_f64
                                                        (f64.store)
                                                        local.set $_rpu_temp_f64
                                                        local.get $_rpu_temp_mem_ptr
                                                        i32.const 0
                                                        i32.add
                                                        local.get $_rpu_temp_f64
                                                        (f64.store)
                                                        local.get $_rpu_temp_mem_ptr
                                                        (local.set $ray_203)
                                                        local.get $fcol_205_x
                                                        local.get $fcol_205_y
                                                        local.get $fcol_205_z
                                                        local.get $fcol_205_w
                                                        (local.get $material_207)
                                                        (i32.const 96)
                                                        (i32.add)
                                                        (f64.load)
                                                        (local.get $material_207)
                                                        (i32.const 104)
                                                        (i32.add)
                                                        (f64.load)
                                                        (local.get $material_207)
                                                        (i32.const 112)
                                                        (i32.add)
                                                        (f64.load)
                                                        (f64.const 1)
                                                        (call $_rpu_vec4_mul_vec4_f64)
                                                        local.get $E_211
                                                        (call $_rpu_vec4_mul_scalar_f64)
                                                        local.get $fcol_205_x
                                                        local.get $fcol_205_y
                                                        local.get $fcol_205_z
                                                        local.get $fcol_205_w
                                                        local.get $color_218_x
                                                        local.get $color_218_y
                                                        local.get $color_218_z
                                                        local.get $color_218_w
                                                        (call $_rpu_vec4_mul_vec4_f64)
                                                        local.get $brdf_220_x
                                                        local.get $brdf_220_y
                                                        local.get $brdf_220_z
                                                        (f64.const 1)
                                                        (call $_rpu_vec4_mul_vec4_f64)
                                                        (call $_rpu_vec4_add_vec4_f64)
                                                        local.set $_rpu_temp_f64
                                                        local.get $tcol_204_w
                                                        local.get $_rpu_temp_f64
                                                        f64.add
                                                        local.set $tcol_204_w
                                                        local.set $_rpu_temp_f64
                                                        local.get $tcol_204_z
                                                        local.get $_rpu_temp_f64
                                                        f64.add
                                                        local.set $tcol_204_z
                                                        local.set $_rpu_temp_f64
                                                        local.get $tcol_204_y
                                                        local.get $_rpu_temp_f64
                                                        f64.add
                                                        local.set $tcol_204_y
                                                        local.set $_rpu_temp_f64
                                                        local.get $tcol_204_x
                                                        local.get $_rpu_temp_f64
                                                        f64.add
                                                        local.set $tcol_204_x
                                                        local.get $color_218_x
                                                        local.get $color_218_y
                                                        local.get $color_218_z
                                                        local.get $color_218_w
                                                        local.set $_rpu_temp_f64
                                                        local.get $fcol_205_w
                                                        local.get $_rpu_temp_f64
                                                        f64.mul
                                                        local.set $fcol_205_w
                                                        local.set $_rpu_temp_f64
                                                        local.get $fcol_205_z
                                                        local.get $_rpu_temp_f64
                                                        f64.mul
                                                        local.set $fcol_205_z
                                                        local.set $_rpu_temp_f64
                                                        local.get $fcol_205_y
                                                        local.get $_rpu_temp_f64
                                                        f64.mul
                                                        local.set $fcol_205_y
                                                        local.set $_rpu_temp_f64
                                                        local.get $fcol_205_x
                                                        local.get $_rpu_temp_f64
                                                        f64.mul
                                                        local.set $fcol_205_x
                                                    )
                                                )
                                                (else
                                                    (block
                                                        (call $random)
                                                        local.set $r2_227
                                                        local.get $normal_208_x
                                                        local.get $normal_208_y
                                                        local.get $normal_208_z
                                                        (f64.const 2)
                                                        (f64.const 3.1415927)
                                                        (f64.mul)
                                                        (call $random)
                                                        (f64.mul)
                                                        local.get $r2_227
                                                        (call $_rpu_vec1_sqrt_f64)
                                                        (f64.const 1)
                                                        local.get $r2_227
                                                        (f64.sub)
                                                        (call $_rpu_vec1_sqrt_f64)
                                                        (call $jitter)
                                                        local.set $d_228_z
                                                        local.set $d_228_y
                                                        local.set $d_228_x
                                                        local.get $hitOrigin_210_x
                                                        local.get $hitOrigin_210_y
                                                        local.get $hitOrigin_210_z
                                                        local.get $normal_208_x
                                                        local.get $normal_208_y
                                                        local.get $normal_208_z
                                                        (local.get $ray_203)
                                                        (i32.const 24)
                                                        (i32.add)
                                                        (f64.load)
                                                        (local.get $ray_203)
                                                        (i32.const 32)
                                                        (i32.add)
                                                        (f64.load)
                                                        (local.get $ray_203)
                                                        (i32.const 40)
                                                        (i32.add)
                                                        (f64.load)
                                                        (local.get $material_207)
                                                        (call $sampleLightE)
                                                        local.set $e_229_z
                                                        local.set $e_229_y
                                                        local.set $e_229_x
                                                        (f64.const 1)
                                                        local.set $E_230
                                                        local.get $hitOrigin_210_x
                                                        local.get $hitOrigin_210_y
                                                        local.get $hitOrigin_210_z
                                                        local.get $d_228_x
                                                        local.get $d_228_y
                                                        local.get $d_228_z
                                                        (i32.const 48)
                                                        (call $malloc)
                                                        (local.set $_rpu_temp_mem_ptr)
                                                        local.set $_rpu_temp_f64
                                                        local.get $_rpu_temp_mem_ptr
                                                        i32.const 40
                                                        i32.add
                                                        local.get $_rpu_temp_f64
                                                        (f64.store)
                                                        local.set $_rpu_temp_f64
                                                        local.get $_rpu_temp_mem_ptr
                                                        i32.const 32
                                                        i32.add
                                                        local.get $_rpu_temp_f64
                                                        (f64.store)
                                                        local.set $_rpu_temp_f64
                                                        local.get $_rpu_temp_mem_ptr
                                                        i32.const 24
                                                        i32.add
                                                        local.get $_rpu_temp_f64
                                                        (f64.store)
                                                        local.set $_rpu_temp_f64
                                                        local.get $_rpu_temp_mem_ptr
                                                        i32.const 16
                                                        i32.add
                                                        local.get $_rpu_temp_f64
                                                        (f64.store)
                                                        local.set $_rpu_temp_f64
                                                        local.get $_rpu_temp_mem_ptr
                                                        i32.const 8
                                                        i32.add
                                                        local.get $_rpu_temp_f64
                                                        (f64.store)
                                                        local.set $_rpu_temp_f64
                                                        local.get $_rpu_temp_mem_ptr
                                                        i32.const 0
                                                        i32.add
                                                        local.get $_rpu_temp_f64
                                                        (f64.store)
                                                        local.get $_rpu_temp_mem_ptr
                                                        (local.set $ray_203)
                                                        local.get $fcol_205_x
                                                        local.get $fcol_205_y
                                                        local.get $fcol_205_z
                                                        local.get $fcol_205_w
                                                        (local.get $material_207)
                                                        (i32.const 96)
                                                        (i32.add)
                                                        (f64.load)
                                                        (local.get $material_207)
                                                        (i32.const 104)
                                                        (i32.add)
                                                        (f64.load)
                                                        (local.get $material_207)
                                                        (i32.const 112)
                                                        (i32.add)
                                                        (f64.load)
                                                        (f64.const 1)
                                                        (call $_rpu_vec4_mul_vec4_f64)
                                                        local.get $E_230
                                                        (call $_rpu_vec4_mul_scalar_f64)
                                                        local.get $fcol_205_x
                                                        local.get $fcol_205_y
                                                        local.get $fcol_205_z
                                                        local.get $fcol_205_w
                                                        local.get $color_218_x
                                                        local.get $color_218_y
                                                        local.get $color_218_z
                                                        local.get $color_218_w
                                                        (call $_rpu_vec4_mul_vec4_f64)
                                                        local.get $e_229_x
                                                        local.get $e_229_y
                                                        local.get $e_229_z
                                                        (f64.const 1)
                                                        (call $_rpu_vec4_mul_vec4_f64)
                                                        (call $_rpu_vec4_add_vec4_f64)
                                                        local.set $_rpu_temp_f64
                                                        local.get $tcol_204_w
                                                        local.get $_rpu_temp_f64
                                                        f64.add
                                                        local.set $tcol_204_w
                                                        local.set $_rpu_temp_f64
                                                        local.get $tcol_204_z
                                                        local.get $_rpu_temp_f64
                                                        f64.add
                                                        local.set $tcol_204_z
                                                        local.set $_rpu_temp_f64
                                                        local.get $tcol_204_y
                                                        local.get $_rpu_temp_f64
                                                        f64.add
                                                        local.set $tcol_204_y
                                                        local.set $_rpu_temp_f64
                                                        local.get $tcol_204_x
                                                        local.get $_rpu_temp_f64
                                                        f64.add
                                                        local.set $tcol_204_x
                                                        local.get $color_218_x
                                                        local.get $color_218_y
                                                        local.get $color_218_z
                                                        local.get $color_218_w
                                                        local.set $_rpu_temp_f64
                                                        local.get $fcol_205_w
                                                        local.get $_rpu_temp_f64
                                                        f64.mul
                                                        local.set $fcol_205_w
                                                        local.set $_rpu_temp_f64
                                                        local.get $fcol_205_z
                                                        local.get $_rpu_temp_f64
                                                        f64.mul
                                                        local.set $fcol_205_z
                                                        local.set $_rpu_temp_f64
                                                        local.get $fcol_205_y
                                                        local.get $_rpu_temp_f64
                                                        f64.mul
                                                        local.set $fcol_205_y
                                                        local.set $_rpu_temp_f64
                                                        local.get $fcol_205_x
                                                        local.get $_rpu_temp_f64
                                                        f64.mul
                                                        local.set $fcol_205_x
                                                    )
                                                )
                                            )
                                        )
                                    )
                                )

                                (local.get $material_207)
                                (i32.const 0)
                                (i32.add)
                                (i64.load)
                                (i64.const 2)
                                (i64.eq)
                                (if
                                    (then
                                        (block
                                            (local.get $material_207)
                                            (i32.const 96)
                                            (i32.add)
                                            (f64.load)
                                            (local.get $material_207)
                                            (i32.const 104)
                                            (i32.add)
                                            (f64.load)
                                            (local.get $material_207)
                                            (i32.const 112)
                                            (i32.add)
                                            (f64.load)
                                            (f64.const 1)
                                            (return)
                                        )
                                    )
                                )
                            )
                        )
                        (else
                            (block
                                (f64.const 0.322)
                                (f64.const 0.322)
                                (f64.const 0.322)
                                (f64.const 1)
                                local.set $backColor_231_w
                                local.set $backColor_231_z
                                local.set $backColor_231_y
                                local.set $backColor_231_x
                                local.get $tcol_204_x
                                local.get $tcol_204_y
                                local.get $tcol_204_z
                                local.get $tcol_204_w
                                local.get $fcol_205_x
                                local.get $fcol_205_y
                                local.get $fcol_205_z
                                local.get $fcol_205_w
                                local.get $backColor_231_x
                                local.get $backColor_231_y
                                local.get $backColor_231_z
                                local.get $backColor_231_w
                                (call $_rpu_vec4_mul_vec4_f64)
                                (call $_rpu_vec4_add_vec4_f64)
                                (return)
                            )
                        )
                    )
                )
                (i64.const 1)
                local.get $depth_206
                i64.add
                local.set $depth_206
                (br 0)
            )
        )
        local.get $tcol_204_x
        local.get $tcol_204_y
        local.get $tcol_204_z
        local.get $tcol_204_w
        (return)
    )

    ;; function 'shader'
    (func $shader (export "shader") (param $coord_234_x f64) (param $coord_234_y f64)(param $resolution_235_x f64) (param $resolution_235_y f64) (result f64 f64 f64 f64)
        (local $uv_236_x f64)
        (local $uv_236_y f64)
        (local $ratio_237 f64)
        (local $pixelSize_238_x f64)
        (local $pixelSize_238_y f64)
        (local $fov_239 f64)
        (local $halfWidth_240 f64)
        (local $halfHeight_241 f64)
        (local $upVector_242_x f64)
        (local $upVector_242_y f64)
        (local $upVector_242_z f64)
        (local $w_243_x f64)
        (local $w_243_y f64)
        (local $w_243_z f64)
        (local $u_244_x f64)
        (local $u_244_y f64)
        (local $u_244_z f64)
        (local $v_245_x f64)
        (local $v_245_y f64)
        (local $v_245_z f64)
        (local $lowerLeft_246_x f64)
        (local $lowerLeft_246_y f64)
        (local $lowerLeft_246_z f64)
        (local $horizontal_247_x f64)
        (local $horizontal_247_y f64)
        (local $horizontal_247_z f64)
        (local $vertical_248_x f64)
        (local $vertical_248_y f64)
        (local $vertical_248_z f64)
        (local $dir_249_x f64)
        (local $dir_249_y f64)
        (local $dir_249_z f64)
        (local $rand_250_x f64)
        (local $rand_250_y f64)
        (local $_rpu_temp_f64 f64)
        (local $_rpu_temp_mem_ptr i32)
        (local $color_251_x f64)
        (local $color_251_y f64)
        (local $color_251_z f64)
        local.get $coord_234_x
        local.get $coord_234_y
        local.get $resolution_235_x
        local.get $resolution_235_y
        (call $_rpu_vec2_div_vec2_f64)
        local.set $uv_236_y
        local.set $uv_236_x
        local.get $resolution_235_x
        local.get $resolution_235_y
        (f64.div)
        local.set $ratio_237
        (f64.const 1)
        (f64.const 1)
        local.get $resolution_235_x
        local.get $resolution_235_y
        (call $_rpu_vec2_div_vec2_f64)
        local.set $pixelSize_238_y
        local.set $pixelSize_238_x
        (f64.const 80)
        local.set $fov_239
        local.get $fov_239
        (call $_rpu_vec1_radians_f64)
        (f64.const 0.5)
        (f64.mul)
        (call $_rpu_vec1_tan_f64)
        local.set $halfWidth_240
        local.get $halfWidth_240
        local.get $ratio_237
        (f64.div)
        local.set $halfHeight_241
        (f64.const 0)
        (f64.const 1)
        (f64.const 0)
        local.set $upVector_242_z
        local.set $upVector_242_y
        local.set $upVector_242_x
        global.get $uOrigin_232_x
        global.get $uOrigin_232_y
        global.get $uOrigin_232_z
        global.get $uLookAt_233_x
        global.get $uLookAt_233_y
        global.get $uLookAt_233_z
        (call $_rpu_vec3_sub_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $w_243_z
        local.set $w_243_y
        local.set $w_243_x
        local.get $upVector_242_x
        local.get $upVector_242_y
        local.get $upVector_242_z
        local.get $w_243_x
        local.get $w_243_y
        local.get $w_243_z
        (call $_rpu_cross_product_f64)
        local.set $u_244_z
        local.set $u_244_y
        local.set $u_244_x
        local.get $w_243_x
        local.get $w_243_y
        local.get $w_243_z
        local.get $u_244_x
        local.get $u_244_y
        local.get $u_244_z
        (call $_rpu_cross_product_f64)
        local.set $v_245_z
        local.set $v_245_y
        local.set $v_245_x
        global.get $uOrigin_232_x
        global.get $uOrigin_232_y
        global.get $uOrigin_232_z
        local.get $halfWidth_240
        local.get $u_244_x
        local.get $u_244_y
        local.get $u_244_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_sub_vec3_f64)
        local.get $halfHeight_241
        local.get $v_245_x
        local.get $v_245_y
        local.get $v_245_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_sub_vec3_f64)
        local.get $w_243_x
        local.get $w_243_y
        local.get $w_243_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $lowerLeft_246_z
        local.set $lowerLeft_246_y
        local.set $lowerLeft_246_x
        local.get $u_244_x
        local.get $u_244_y
        local.get $u_244_z
        local.get $halfWidth_240
        (call $_rpu_vec3_mul_scalar_f64)
        (f64.const 2)
        (call $_rpu_vec3_mul_scalar_f64)
        local.set $horizontal_247_z
        local.set $horizontal_247_y
        local.set $horizontal_247_x
        local.get $v_245_x
        local.get $v_245_y
        local.get $v_245_z
        local.get $halfHeight_241
        (call $_rpu_vec3_mul_scalar_f64)
        (f64.const 2)
        (call $_rpu_vec3_mul_scalar_f64)
        local.set $vertical_248_z
        local.set $vertical_248_y
        local.set $vertical_248_x
        local.get $lowerLeft_246_x
        local.get $lowerLeft_246_y
        local.get $lowerLeft_246_z
        global.get $uOrigin_232_x
        global.get $uOrigin_232_y
        global.get $uOrigin_232_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $dir_249_z
        local.set $dir_249_y
        local.set $dir_249_x
        (call $rand2)
        local.set $rand_250_y
        local.set $rand_250_x
        local.get $horizontal_247_x
        local.get $horizontal_247_y
        local.get $horizontal_247_z
        local.get $pixelSize_238_x
        local.get $rand_250_x
        (f64.mul)
        local.get $uv_236_x
        (f64.add)
        (call $_rpu_vec3_mul_scalar_f64)
        local.set $_rpu_temp_f64
        local.get $dir_249_z
        local.get $_rpu_temp_f64
        f64.add
        local.set $dir_249_z
        local.set $_rpu_temp_f64
        local.get $dir_249_y
        local.get $_rpu_temp_f64
        f64.add
        local.set $dir_249_y
        local.set $_rpu_temp_f64
        local.get $dir_249_x
        local.get $_rpu_temp_f64
        f64.add
        local.set $dir_249_x
        local.get $vertical_248_x
        local.get $vertical_248_y
        local.get $vertical_248_z
        local.get $pixelSize_238_y
        local.get $rand_250_y
        (f64.mul)
        local.get $uv_236_y
        (f64.add)
        (call $_rpu_vec3_mul_scalar_f64)
        local.set $_rpu_temp_f64
        local.get $dir_249_z
        local.get $_rpu_temp_f64
        f64.add
        local.set $dir_249_z
        local.set $_rpu_temp_f64
        local.get $dir_249_y
        local.get $_rpu_temp_f64
        f64.add
        local.set $dir_249_y
        local.set $_rpu_temp_f64
        local.get $dir_249_x
        local.get $_rpu_temp_f64
        f64.add
        local.set $dir_249_x
        global.get $uOrigin_232_x
        global.get $uOrigin_232_y
        global.get $uOrigin_232_z
        local.get $dir_249_x
        local.get $dir_249_y
        local.get $dir_249_z
        (call $_rpu_normalize_vec3_f64)
        (i32.const 48)
        (call $malloc)
        (local.set $_rpu_temp_mem_ptr)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 40
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 32
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 24
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 16
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 8
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $_rpu_temp_mem_ptr
        i32.const 0
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.get $_rpu_temp_mem_ptr
        (call $getColor)
        (local.set $_rpu_temp_f64)
        (i32.const 24)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 16)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 8)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (i32.const 0)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (i32.const 0)
        (f64.load)
        (i32.const 8)
        (f64.load)
        (i32.const 16)
        (f64.load)
        local.set $color_251_z
        local.set $color_251_y
        local.set $color_251_x
        local.get $color_251_x
        local.get $color_251_y
        local.get $color_251_z
        (f64.const 1)
        (return)
    )
)
