(module
    (import "env" "_rpu_sin" (func $_rpu_sin (param f64) (result f64)))
    (import "env" "_rpu_cos" (func $_rpu_cos (param f64) (result f64)))
    (import "env" "_rpu_tan" (func $_rpu_tan (param f64) (result f64)))
    (import "env" "_rpu_atan" (func $_rpu_atan (param f64) (result f64)))
    (import "env" "_rpu_sign" (func $_rpu_sign (param f64) (result f64)))
    (import "env" "_rpu_radians" (func $_rpu_radians (param f64) (result f64)))
    (import "env" "_rpu_min" (func $_rpu_min (param f64) (param f64) (result f64)))
    (import "env" "_rpu_max" (func $_rpu_max (param f64) (param f64) (result f64)))
    (import "env" "_rpu_pow" (func $_rpu_pow (param f64) (param f64) (result f64)))
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

    ;; vec1 cos
    (func $_rpu_vec1_cos_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_cos))

    ;; vec1 sin
    (func $_rpu_vec1_sin_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_sin))

    ;; vec1 neg
    (func $_rpu_vec1_neg_f64  (param $x f64)  (result f64)
        local.get $x
        f64.neg)

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

    ;; vec1 max
    (func $_rpu_vec1_max_f64  (param $x f64)  (param $scalar f64)  (result f64)
        local.get $x
        local.get $scalar
        (call $_rpu_max))

    ;; vec1 min
    (func $_rpu_vec1_min_f64  (param $x f64)  (param $scalar f64)  (result f64)
        local.get $x
        local.get $scalar
        (call $_rpu_min))

    ;; vec1 clamp
    (func $_rpu_vec1_clamp_f64_f64  (param $x f64)  (param $scalar f64) (param $scalar2 f64) (result f64)
        local.get $x
        local.get $scalar
        local.get $scalar2
        (call $_rpu_clamp))

    ;; vec3 div vec3 (f64)
    (func $_rpu_vec3_div_vec3_f64
        (param $vec2l_x f64)
        (param $vec2l_y f64)
        (param $vec2l_z f64)
        (param $vec2r_x f64)
        (param $vec2r_y f64)
        (param $vec2r_z f64)
        (result f64 f64 f64)

        (f64.div
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        (f64.div
            (local.get $vec2l_y)
            (local.get $vec2r_y)
        )

        (f64.div
            (local.get $vec2l_z)
            (local.get $vec2r_z)
        )
    )

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

    ;; vec1 abs
    (func $_rpu_vec1_abs_f64  (param $x f64)  (result f64)
        local.get $x
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

    ;; vec1 sqrt
    (func $_rpu_vec1_sqrt_f64  (param $x f64)  (result f64)
        local.get $x
        f64.sqrt)

    ;; vec2 abs
    (func $_rpu_vec2_abs_f64  (param $x f64)  (param $y f64)  (result f64 f64)
        local.get $x
        f64.abs
        local.get $y
        f64.abs)

    ;; vec2 max
    (func $_rpu_vec2_max_f64  (param $x f64)  (param $y f64)  (param $scalar f64)  (result f64 f64)
        local.get $x
        local.get $scalar
        (call $_rpu_max)
        local.get $y
        local.get $scalar
        (call $_rpu_max))

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

    ;; vec3 fract
    (func $_rpu_vec3_fract_f64  (param $x f64)  (param $y f64)  (param $z f64)  (result f64 f64 f64)
        local.get $x
        (call $_rpu_fract)
        local.get $y
        (call $_rpu_fract)
        local.get $z
        (call $_rpu_fract))

    ;; vec3 add scalar (f64)
    (func $_rpu_vec3_add_scalar_f64
        (param $vec3_x f64)    ;; x component of vec3
        (param $vec3_y f64)    ;; y component of vec3
        (param $vec3_z f64)    ;; z component of vec3
        (param $scalar f64)    ;; Scalar
        (result f64 f64 f64)       ;; Return three f64 results, the new x, y and z components

        ;; Calculate the new x component and return it
        (f64.add
            (local.get $vec3_x)  ;; Get the x component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new y component and return it
        (f64.add
            (local.get $vec3_y)  ;; Get the y component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new z component and return it
        (f64.add
            (local.get $vec3_z)  ;; Get the z component
            (local.get $scalar)  ;; Get the scalar
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

    ;; vec1 fract
    (func $_rpu_vec1_fract_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_fract))

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

    ;; vec1 pow
    (func $_rpu_vec1_pow_f64  (param $x f64)  (param $scalar f64)  (result f64)
        local.get $x
        local.get $scalar
        (call $_rpu_pow))

    ;; vec1 radians
    (func $_rpu_vec1_radians_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_radians))

    ;; vec1 tan
    (func $_rpu_vec1_tan_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_tan))

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

    ;; vec1 sign
    (func $_rpu_vec1_sign_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_sign))

    ;; vec1 atan
    (func $_rpu_vec1_atan_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_atan))

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

    ;; vec3 pow
    (func $_rpu_vec3_pow_f64  (param $x f64)  (param $y f64)  (param $z f64)  (param $scalar f64)  (result f64 f64 f64)
        local.get $x
        local.get $scalar
        (call $_rpu_pow)
        local.get $y
        local.get $scalar
        (call $_rpu_pow)
        local.get $z
        local.get $scalar
        (call $_rpu_pow))

    ;; function 'opU'
    (func $opU (param $o1_0_x f64) (param $o1_0_y f64)(param $o2_1_x f64) (param $o2_1_y f64) (result f64 f64)
        (local $$_rpu_ternary_0_x f64)
        (local $$_rpu_ternary_0_y f64)

        local.get $o1_0_x
        local.get $o2_1_x
        (f64.lt)
        (if
            (then
                local.get $o1_0_x
                local.get $o1_0_y
                (local.set $$_rpu_ternary_0_y)
                (local.set $$_rpu_ternary_0_x)
            )
            (else
                local.get $o2_1_x
                local.get $o2_1_y
                (local.set $$_rpu_ternary_0_y)
                (local.set $$_rpu_ternary_0_x)
            )
        )
        (local.get $$_rpu_ternary_0_x)
        (local.get $$_rpu_ternary_0_y)
        (return)
    )

    ;; function 'opTwist'
    (func $opTwist (param $p_2_x f64) (param $p_2_y f64) (param $p_2_z f64)(param $k_3 f64) (result f64 f64 f64)
        (local $c_4 f64)
        (local $s_5 f64)
        (local $m_6_x f64)
        (local $m_6_y f64)
        (local $m_6_z f64)
        (local $m_6_w f64)
        (local $q_7_x f64)
        (local $q_7_y f64)
        (local $q_7_z f64)
        local.get $k_3
        local.get $p_2_y
        (f64.mul)
        (call $_rpu_vec1_cos_f64)
        local.set $c_4
        local.get $k_3
        local.get $p_2_y
        (f64.mul)
        (call $_rpu_vec1_sin_f64)
        local.set $s_5
        local.get $c_4
        local.get $s_5
        (call $_rpu_vec1_neg_f64)
        local.get $s_5
        local.get $c_4
        (local.set $m_6_w)
        (local.set $m_6_z)
        (local.set $m_6_y)
        (local.set $m_6_x)
        (local.get $m_6_x)
        (local.get $m_6_y)
        (local.get $m_6_z)
        (local.get $m_6_w)
        local.get $p_2_x
        local.get $p_2_z
        (call $_rpu_mat2_mul_vec2_f64)
        local.get $p_2_y
        local.set $q_7_z
        local.set $q_7_y
        local.set $q_7_x
        local.get $q_7_x
        local.get $q_7_y
        local.get $q_7_z
        (return)
    )

    ;; function 'sdBox'
    (func $sdBox (param $p_8_x f64) (param $p_8_y f64) (param $p_8_z f64)(param $s_9_x f64) (param $s_9_y f64) (param $s_9_z f64) (result f64)
        local.get $p_8_x
        local.get $p_8_y
        local.get $p_8_z
        (call $_rpu_vec3_abs_f64)
        local.get $s_9_x
        local.get $s_9_y
        local.get $s_9_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $p_8_z
        local.set $p_8_y
        local.set $p_8_x
        local.get $p_8_x
        local.get $p_8_y
        local.get $p_8_z
        (f64.const 0)
        (call $_rpu_vec3_max_f64)
        (call $_rpu_vec3_length_f64)
        local.get $p_8_x
        local.get $p_8_y
        local.get $p_8_z
        (call $_rpu_vec1_max_f64)
        (call $_rpu_vec1_max_f64)
        (f64.const 0)
        (call $_rpu_vec1_min_f64)
        (f64.add)
        (return)
    )

    ;; function 'sdSphere'
    (func $sdSphere (param $p_10_x f64) (param $p_10_y f64) (param $p_10_z f64)(param $r_11 f64) (result f64)
        local.get $p_10_x
        local.get $p_10_y
        local.get $p_10_z
        (call $_rpu_vec3_length_f64)
        local.get $r_11
        (f64.sub)
        (return)
    )

    ;; function 'sdOctahedron'
    (func $sdOctahedron (param $p_12_x f64) (param $p_12_y f64) (param $p_12_z f64)(param $s_13 f64) (result f64)
        (local $m_14 f64)
        (local $q_15_x f64)
        (local $q_15_y f64)
        (local $q_15_z f64)
        (local $k_16 f64)
        local.get $p_12_x
        local.get $p_12_y
        local.get $p_12_z
        (call $_rpu_vec3_abs_f64)
        local.set $p_12_z
        local.set $p_12_y
        local.set $p_12_x
        local.get $p_12_x
        local.get $p_12_y
        (f64.add)
        local.get $p_12_z
        (f64.add)
        local.get $s_13
        (f64.sub)
        local.set $m_14
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $q_15_z
        local.set $q_15_y
        local.set $q_15_x

        (f64.const 3)
        local.get $p_12_x
        (f64.mul)
        local.get $m_14
        (f64.lt)
        (if
            (then
                local.get $p_12_x
                local.get $p_12_y
                local.get $p_12_z
                local.set $q_15_z
                local.set $q_15_y
                local.set $q_15_x
            )
            (else

                (f64.const 3)
                local.get $p_12_y
                (f64.mul)
                local.get $m_14
                (f64.lt)
                (if
                    (then
                        local.get $p_12_y
                        local.get $p_12_z
                        local.get $p_12_x
                        local.set $q_15_z
                        local.set $q_15_y
                        local.set $q_15_x
                    )
                    (else

                        (f64.const 3)
                        local.get $p_12_z
                        (f64.mul)
                        local.get $m_14
                        (f64.lt)
                        (if
                            (then
                                local.get $p_12_z
                                local.get $p_12_x
                                local.get $p_12_y
                                local.set $q_15_z
                                local.set $q_15_y
                                local.set $q_15_x
                            )
                            (else
                                local.get $m_14
                                (f64.const 0.57735026)
                                (f64.mul)
                                (return)
                            )
                        )
                    )
                )
            )
        )
        (f64.const 0.5)
        local.get $q_15_z
        local.get $q_15_y
        (f64.sub)
        local.get $s_13
        (f64.add)
        (f64.mul)
        (f64.const 0)
        local.get $s_13
        (call $_rpu_vec1_clamp_f64_f64)
        local.set $k_16
        local.get $q_15_x
        local.get $q_15_y
        local.get $s_13
        (f64.sub)
        local.get $k_16
        (f64.add)
        local.get $q_15_z
        local.get $k_16
        (f64.sub)
        (call $_rpu_vec3_length_f64)
        (return)
    )

    ;; function 'sdEllipsoid'
    (func $sdEllipsoid (param $p_17_x f64) (param $p_17_y f64) (param $p_17_z f64)(param $r_18_x f64) (param $r_18_y f64) (param $r_18_z f64) (result f64)
        (local $k0_19 f64)
        (local $k1_20 f64)
        local.get $p_17_x
        local.get $p_17_y
        local.get $p_17_z
        local.get $r_18_x
        local.get $r_18_y
        local.get $r_18_z
        (call $_rpu_vec3_div_vec3_f64)
        (call $_rpu_vec3_length_f64)
        local.set $k0_19
        local.get $p_17_x
        local.get $p_17_y
        local.get $p_17_z
        local.get $r_18_x
        local.get $r_18_y
        local.get $r_18_z
        local.get $r_18_x
        local.get $r_18_y
        local.get $r_18_z
        (call $_rpu_vec3_mul_vec3_f64)
        (call $_rpu_vec3_div_vec3_f64)
        (call $_rpu_vec3_length_f64)
        local.set $k1_20
        local.get $k0_19
        local.get $k0_19
        (f64.const 1)
        (f64.sub)
        (f64.mul)
        local.get $k1_20
        (f64.div)
        (return)
    )

    ;; function 'sdCappedCone'
    (func $sdCappedCone (param $p_21_x f64) (param $p_21_y f64) (param $p_21_z f64)(param $h_22 f64)(param $r1_23 f64)(param $r2_24 f64) (result f64)
        (local $q_25_x f64)
        (local $q_25_y f64)
        (local $k1_26_x f64)
        (local $k1_26_y f64)
        (local $k2_27_x f64)
        (local $k2_27_y f64)
        (local $_rpu_ternary_1 f64)
        (local $ca_28_x f64)
        (local $ca_28_y f64)
        (local $cb_29_x f64)
        (local $cb_29_y f64)
        (local $_rpu_ternary_2 f64)
        (local $s_30 f64)
        local.get $p_21_x
        local.get $p_21_z
        (call $_rpu_vec2_length_f64)
        local.get $p_21_y
        local.set $q_25_y
        local.set $q_25_x
        local.get $r2_24
        local.get $h_22
        local.set $k1_26_y
        local.set $k1_26_x
        local.get $r2_24
        local.get $r1_23
        (f64.sub)
        (f64.const 2)
        local.get $h_22
        (f64.mul)
        local.set $k2_27_y
        local.set $k2_27_x
        local.get $q_25_x
        local.get $q_25_x

        local.get $q_25_y
        (f64.const 0)
        (f64.lt)
        (if
            (then
                local.get $r1_23
                (local.set $_rpu_ternary_1)
            )
            (else
                local.get $r2_24
                (local.set $_rpu_ternary_1)
            )
        )
        (local.get $_rpu_ternary_1)
        (call $_rpu_vec1_min_f64)
        (f64.sub)
        local.get $q_25_y
        (call $_rpu_vec1_abs_f64)
        local.get $h_22
        (f64.sub)
        local.set $ca_28_y
        local.set $ca_28_x
        local.get $q_25_x
        local.get $q_25_y
        local.get $k1_26_x
        local.get $k1_26_y
        (call $_rpu_vec2_sub_vec2_f64)
        local.get $k2_27_x
        local.get $k2_27_y
        local.get $k1_26_x
        local.get $k1_26_y
        local.get $q_25_x
        local.get $q_25_y
        (call $_rpu_vec2_sub_vec2_f64)
        local.get $k2_27_x
        local.get $k2_27_y
        (call $_rpu_dot_product_vec2_f64)
        local.get $k2_27_x
        local.get $k2_27_y
        local.get $k2_27_x
        local.get $k2_27_y
        (call $_rpu_dot_product_vec2_f64)
        (f64.div)
        (f64.const 0)
        (f64.const 1)
        (call $_rpu_vec1_clamp_f64_f64)
        (call $_rpu_vec2_mul_scalar_f64)
        (call $_rpu_vec2_add_vec2_f64)
        local.set $cb_29_y
        local.set $cb_29_x

        local.get $cb_29_x
        (f64.const 0)
        (f64.lt)
        local.get $ca_28_y
        (f64.const 0)
        (f64.lt)
        (i32.and)
        (if
            (then
                (f64.const 1)
                (call $_rpu_vec1_neg_f64)
                (local.set $_rpu_ternary_2)
            )
            (else
                (f64.const 1)
                (local.set $_rpu_ternary_2)
            )
        )
        (local.get $_rpu_ternary_2)
        local.set $s_30
        local.get $s_30
        local.get $ca_28_x
        local.get $ca_28_y
        local.get $ca_28_x
        local.get $ca_28_y
        (call $_rpu_dot_product_vec2_f64)
        local.get $cb_29_x
        local.get $cb_29_y
        local.get $cb_29_x
        local.get $cb_29_y
        (call $_rpu_dot_product_vec2_f64)
        (call $_rpu_vec1_min_f64)
        (call $_rpu_vec1_sqrt_f64)
        (f64.mul)
        (return)
    )

    ;; function 'sdCappedCylinder'
    (func $sdCappedCylinder (param $p_31_x f64) (param $p_31_y f64) (param $p_31_z f64)(param $h_32 f64)(param $r_33 f64) (result f64)
        (local $d_34_x f64)
        (local $d_34_y f64)
        local.get $p_31_x
        local.get $p_31_z
        (call $_rpu_vec2_length_f64)
        local.get $p_31_y
        (call $_rpu_vec2_abs_f64)
        local.get $r_33
        local.get $h_32
        (call $_rpu_vec2_sub_vec2_f64)
        local.set $d_34_y
        local.set $d_34_x
        local.get $d_34_x
        local.get $d_34_y
        (call $_rpu_vec1_max_f64)
        (f64.const 0)
        (call $_rpu_vec1_min_f64)
        local.get $d_34_x
        local.get $d_34_y
        (f64.const 0)
        (call $_rpu_vec2_max_f64)
        (call $_rpu_vec2_length_f64)
        (f64.add)
        (return)
    )

    ;; function 'sdPlane'
    (func $sdPlane (param $p_35_x f64) (param $p_35_y f64) (param $p_35_z f64)(param $n_36_x f64) (param $n_36_y f64) (param $n_36_z f64) (param $n_36_w f64) (result f64)
        local.get $p_35_x
        local.get $p_35_y
        local.get $p_35_z
        local.get $n_36_x
        local.get $n_36_y
        local.get $n_36_z
        (call $_rpu_dot_product_vec3_f64)
        local.get $n_36_w
        (f64.add)
        (return)
    )

    ;; function 'reflect'
    (func $reflect (param $incident_37_x f64) (param $incident_37_y f64) (param $incident_37_z f64)(param $normal_38_x f64) (param $normal_38_y f64) (param $normal_38_z f64) (result f64 f64 f64)
        local.get $incident_37_x
        local.get $incident_37_y
        local.get $incident_37_z
        (f64.const 2)
        local.get $normal_38_x
        local.get $normal_38_y
        local.get $normal_38_z
        local.get $incident_37_x
        local.get $incident_37_y
        local.get $incident_37_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.mul)
        local.get $normal_38_x
        local.get $normal_38_y
        local.get $normal_38_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_sub_vec3_f64)
        (return)
    )

    ;; function 'hash31'
    (func $hash31 (param $p_39 f64) (result f64 f64 f64)
        (local $p3_40_x f64)
        (local $p3_40_y f64)
        (local $p3_40_z f64)
        (local $_rpu_temp_f64 f64)
        local.get $p_39
        local.get $p_39
        local.get $p_39
        (f64.const 0.1031)
        (f64.const 0.103)
        (f64.const 0.0973)
        (call $_rpu_vec3_mul_vec3_f64)
        (call $_rpu_vec3_fract_f64)
        local.set $p3_40_z
        local.set $p3_40_y
        local.set $p3_40_x
        local.get $p3_40_x
        local.get $p3_40_y
        local.get $p3_40_z
        local.get $p3_40_y
        local.get $p3_40_z
        local.get $p3_40_x
        (f64.const 33.33)
        (call $_rpu_vec3_add_scalar_f64)
        (call $_rpu_dot_product_vec3_f64)
        local.get $p3_40_x
        local.get $p3_40_y
        local.get $p3_40_z
        local.get $p3_40_y
        local.get $p3_40_z
        local.get $p3_40_x
        (f64.const 33.33)
        (call $_rpu_vec3_add_scalar_f64)
        (call $_rpu_dot_product_vec3_f64)
        local.get $p3_40_x
        local.get $p3_40_y
        local.get $p3_40_z
        local.get $p3_40_y
        local.get $p3_40_z
        local.get $p3_40_x
        (f64.const 33.33)
        (call $_rpu_vec3_add_scalar_f64)
        (call $_rpu_dot_product_vec3_f64)
        local.set $_rpu_temp_f64
        local.get $p3_40_z
        local.get $_rpu_temp_f64
        f64.add
        local.set $p3_40_z
        local.set $_rpu_temp_f64
        local.get $p3_40_y
        local.get $_rpu_temp_f64
        f64.add
        local.set $p3_40_y
        local.set $_rpu_temp_f64
        local.get $p3_40_x
        local.get $_rpu_temp_f64
        f64.add
        local.set $p3_40_x
        local.get $p3_40_x
        local.get $p3_40_x
        local.get $p3_40_y
        local.get $p3_40_y
        local.get $p3_40_z
        local.get $p3_40_z
        (call $_rpu_vec3_add_vec3_f64)
        local.get $p3_40_z
        local.get $p3_40_y
        local.get $p3_40_x
        (call $_rpu_vec3_mul_vec3_f64)
        (call $_rpu_vec3_fract_f64)
        (return)
    )

    ;; function 'getDist'
    (func $getDist (param $p_41_x f64) (param $p_41_y f64) (param $p_41_z f64) (result f64 f64)
        (local $r_42_x f64)
        (local $r_42_y f64)
        (local $plane_43_x f64)
        (local $plane_43_y f64)
        (local $sphere1_44_x f64)
        (local $sphere1_44_y f64)
        (local $sphere2_45_x f64)
        (local $sphere2_45_y f64)
        (local $sphere3_46_x f64)
        (local $sphere3_46_y f64)
        (local $field_47 f64)
        (local $a_48 f64)
        (local $b_49 f64)
        (local $rand1_50_x f64)
        (local $rand1_50_y f64)
        (local $rand1_50_z f64)
        (local $center_51_x f64)
        (local $center_51_y f64)
        (local $center_51_z f64)
        (local $id_52 f64)
        (local $matId_53 f64)
        (local $shape_54_x f64)
        (local $shape_54_y f64)
        (local $shape_55_x f64)
        (local $shape_55_y f64)
        (local $shape_56_x f64)
        (local $shape_56_y f64)
        (local $shape_57_x f64)
        (local $shape_57_y f64)
        (local $shape_58_x f64)
        (local $shape_58_y f64)
        (f64.const 1000)
        (f64.const 0)
        local.set $r_42_y
        local.set $r_42_x

        local.get $p_41_y
        (f64.const 2.1)
        (f64.lt)
        (if
            (then
                (block
                    local.get $p_41_x
                    local.get $p_41_y
                    local.get $p_41_z
                    (f64.const 0)
                    (f64.const 1)
                    (f64.const 0)
                    (f64.const 0)
                    (call $sdPlane)
                    (f64.const 0)
                    local.set $plane_43_y
                    local.set $plane_43_x
                    local.get $r_42_x
                    local.get $r_42_y
                    local.get $plane_43_x
                    local.get $plane_43_y
                    (call $opU)
                    local.set $r_42_y
                    local.set $r_42_x
                    local.get $p_41_x
                    local.get $p_41_y
                    local.get $p_41_z
                    (f64.const 4)
                    (call $_rpu_vec1_neg_f64)
                    (f64.const 1)
                    (f64.const 0)
                    (call $_rpu_vec3_sub_vec3_f64)
                    (f64.const 1)
                    (call $sdSphere)
                    (f64.const 1)
                    local.set $sphere1_44_y
                    local.set $sphere1_44_x
                    local.get $p_41_x
                    local.get $p_41_y
                    local.get $p_41_z
                    (f64.const 0)
                    (f64.const 1)
                    (f64.const 0)
                    (call $_rpu_vec3_sub_vec3_f64)
                    (f64.const 1)
                    (call $sdSphere)
                    (f64.const 2)
                    local.set $sphere2_45_y
                    local.set $sphere2_45_x
                    local.get $p_41_x
                    local.get $p_41_y
                    local.get $p_41_z
                    (f64.const 4)
                    (f64.const 1)
                    (f64.const 0)
                    (call $_rpu_vec3_sub_vec3_f64)
                    (f64.const 1)
                    (call $sdSphere)
                    (f64.const 3)
                    local.set $sphere3_46_y
                    local.set $sphere3_46_x
                    local.get $r_42_x
                    local.get $r_42_y
                    local.get $sphere1_44_x
                    local.get $sphere1_44_y
                    (call $opU)
                    local.set $r_42_y
                    local.set $r_42_x
                    local.get $r_42_x
                    local.get $r_42_y
                    local.get $sphere2_45_x
                    local.get $sphere2_45_y
                    (call $opU)
                    local.set $r_42_y
                    local.set $r_42_x
                    local.get $r_42_x
                    local.get $r_42_y
                    local.get $sphere3_46_x
                    local.get $sphere3_46_y
                    (call $opU)
                    local.set $r_42_y
                    local.set $r_42_x
                )
            )
        )

        local.get $p_41_y
        (f64.const 0.6)
        (f64.lt)
        (if
            (then
                (block
                    (f64.const 8.5)
                    local.set $field_47

                    local.get $field_47
                    (call $_rpu_vec1_neg_f64)
                    local.set $a_48
                    (block
                        (loop
                            local.get $a_48
                            local.get $field_47
                            (f64.lt)
                            (i32.eqz)
                            (br_if 1)
                            (block

                                local.get $field_47
                                (call $_rpu_vec1_neg_f64)
                                local.set $b_49
                                (block
                                    (loop
                                        local.get $b_49
                                        local.get $field_47
                                        (f64.lt)
                                        (i32.eqz)
                                        (br_if 1)
                                        (block
                                            local.get $a_48
                                            local.get $b_49
                                            (f64.mul)
                                            (f64.const 3214.2)
                                            (f64.add)
                                            (call $hash31)
                                            local.set $rand1_50_z
                                            local.set $rand1_50_y
                                            local.set $rand1_50_x
                                            local.get $a_48
                                            (f64.const 14.1)
                                            local.get $rand1_50_x
                                            (f64.mul)
                                            (f64.add)
                                            (f64.const 0.2)
                                            local.get $b_49
                                            (f64.const 11.9)
                                            local.get $rand1_50_y
                                            (f64.mul)
                                            (f64.add)
                                            local.set $center_51_z
                                            local.set $center_51_y
                                            local.set $center_51_x
                                            local.get $rand1_50_z
                                            (f64.const 3.3)
                                            (f64.mul)
                                            (call $_rpu_vec1_fract_f64)
                                            local.set $id_52
                                            local.get $rand1_50_z
                                            local.set $matId_53

                                            local.get $id_52
                                            (f64.const 0.85)
                                            (f64.gt)
                                            (if
                                                (then
                                                    (block
                                                        local.get $p_41_x
                                                        local.get $p_41_y
                                                        local.get $p_41_z
                                                        local.get $center_51_x
                                                        local.get $center_51_y
                                                        local.get $center_51_z
                                                        (call $_rpu_vec3_sub_vec3_f64)
                                                        (f64.const 0.2)
                                                        (call $sdOctahedron)
                                                        local.get $matId_53
                                                        local.set $shape_54_y
                                                        local.set $shape_54_x
                                                        local.get $r_42_x
                                                        local.get $r_42_y
                                                        local.get $shape_54_x
                                                        local.get $shape_54_y
                                                        (call $opU)
                                                        local.set $r_42_y
                                                        local.set $r_42_x
                                                    )
                                                )
                                                (else

                                                    local.get $id_52
                                                    (f64.const 0.75)
                                                    (f64.gt)
                                                    (if
                                                        (then
                                                            (block
                                                                local.get $p_41_x
                                                                local.get $p_41_y
                                                                local.get $p_41_z
                                                                local.get $center_51_x
                                                                local.get $center_51_y
                                                                local.get $center_51_z
                                                                (call $_rpu_vec3_sub_vec3_f64)
                                                                (f64.const 0.1)
                                                                (f64.const 0.2)
                                                                (f64.const 0.1)
                                                                (call $sdEllipsoid)
                                                                local.get $matId_53
                                                                local.set $shape_55_y
                                                                local.set $shape_55_x
                                                                local.get $r_42_x
                                                                local.get $r_42_y
                                                                local.get $shape_55_x
                                                                local.get $shape_55_y
                                                                (call $opU)
                                                                local.set $r_42_y
                                                                local.set $r_42_x
                                                            )
                                                        )
                                                        (else

                                                            local.get $id_52
                                                            (f64.const 0.55)
                                                            (f64.gt)
                                                            (if
                                                                (then
                                                                    (block
                                                                        local.get $p_41_x
                                                                        local.get $p_41_y
                                                                        local.get $p_41_z
                                                                        local.get $center_51_x
                                                                        local.get $center_51_y
                                                                        local.get $center_51_z
                                                                        (call $_rpu_vec3_sub_vec3_f64)
                                                                        (f64.const 0.2)
                                                                        (f64.const 0.2)
                                                                        (f64.const 0)
                                                                        (call $sdCappedCone)
                                                                        local.get $matId_53
                                                                        local.set $shape_56_y
                                                                        local.set $shape_56_x
                                                                        local.get $r_42_x
                                                                        local.get $r_42_y
                                                                        local.get $shape_56_x
                                                                        local.get $shape_56_y
                                                                        (call $opU)
                                                                        local.set $r_42_y
                                                                        local.set $r_42_x
                                                                    )
                                                                )
                                                                (else

                                                                    local.get $id_52
                                                                    (f64.const 0.35)
                                                                    (f64.gt)
                                                                    (if
                                                                        (then
                                                                            (block
                                                                                local.get $p_41_x
                                                                                local.get $p_41_y
                                                                                local.get $p_41_z
                                                                                local.get $center_51_x
                                                                                local.get $center_51_y
                                                                                local.get $center_51_z
                                                                                (call $_rpu_vec3_sub_vec3_f64)
                                                                                (f64.const 0.2)
                                                                                (f64.const 0.2)
                                                                                (f64.const 0.2)
                                                                                (call $sdBox)
                                                                                local.get $matId_53
                                                                                local.set $shape_57_y
                                                                                local.set $shape_57_x
                                                                                local.get $r_42_x
                                                                                local.get $r_42_y
                                                                                local.get $shape_57_x
                                                                                local.get $shape_57_y
                                                                                (call $opU)
                                                                                local.set $r_42_y
                                                                                local.set $r_42_x
                                                                            )
                                                                        )
                                                                        (else
                                                                            (block
                                                                                local.get $p_41_x
                                                                                local.get $p_41_y
                                                                                local.get $p_41_z
                                                                                local.get $center_51_x
                                                                                local.get $center_51_y
                                                                                local.get $center_51_z
                                                                                (call $_rpu_vec3_sub_vec3_f64)
                                                                                (f64.const 0.2)
                                                                                (call $sdSphere)
                                                                                local.get $matId_53
                                                                                local.set $shape_58_y
                                                                                local.set $shape_58_x
                                                                                local.get $r_42_x
                                                                                local.get $r_42_y
                                                                                local.get $shape_58_x
                                                                                local.get $shape_58_y
                                                                                (call $opU)
                                                                                local.set $r_42_y
                                                                                local.set $r_42_x
                                                                            )
                                                                        )
                                                                    )
                                                                )
                                                            )
                                                        )
                                                    )
                                                )
                                            )
                                        )
                                        (f64.const 1)
                                        local.get $b_49
                                        f64.add
                                        local.set $b_49
                                        (br 0)
                                    )
                                )
                            )
                            (f64.const 1)
                            local.get $a_48
                            f64.add
                            local.set $a_48
                            (br 0)
                        )
                    )
                )
            )
        )
        local.get $r_42_x
        local.get $r_42_y
        (return)
    )

    ;; function 'getMaterial'
    (func $getMaterial (param $id_59 f64) (result i32)
        (local $_rpu_temp_mem_ptr i32)
        (local $_rpu_temp_f64 f64)
        (local $_rpu_temp_i64 i64)
        (local $d_60 f64)
        (local $c_61_x f64)
        (local $c_61_y f64)
        (local $c_61_z f64)

        local.get $id_59
        (f64.const 0)
        (f64.eq)
        (if
            (then
                (block
                    (i64.const 1)
                    (f64.const 0.5)
                    (f64.const 0.5)
                    (f64.const 0.5)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0.5)
                    (f64.const 0)
                    (f64.const 0)
                    (f64.const 0)
                    (i32.const 88)
                    (call $malloc)
                    (local.set $_rpu_temp_mem_ptr)
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
                    (return)
                )
            )
            (else

                local.get $id_59
                (f64.const 1)
                (f64.eq)
                (if
                    (then
                        (block
                            (i64.const 0)
                            (f64.const 1)
                            (f64.const 1)
                            (f64.const 1)
                            (f64.const 0)
                            (f64.const 0)
                            (f64.const 0)
                            (f64.const 0.5)
                            (f64.const 0)
                            (f64.const 0)
                            (f64.const 0)
                            (i32.const 88)
                            (call $malloc)
                            (local.set $_rpu_temp_mem_ptr)
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
                            (return)
                        )
                    )
                    (else

                        local.get $id_59
                        (f64.const 2)
                        (f64.eq)
                        (if
                            (then
                                (block
                                    (i64.const 2)
                                    (f64.const 1)
                                    (f64.const 1)
                                    (f64.const 1)
                                    (f64.const 0)
                                    (f64.const 0)
                                    (f64.const 0)
                                    (f64.const 0.5)
                                    (f64.const 0)
                                    (f64.const 0)
                                    (f64.const 1.5)
                                    (i32.const 88)
                                    (call $malloc)
                                    (local.set $_rpu_temp_mem_ptr)
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
                                    (return)
                                )
                            )
                            (else

                                local.get $id_59
                                (f64.const 3)
                                (f64.eq)
                                (if
                                    (then
                                        (block
                                            (i64.const 1)
                                            (f64.const 1)
                                            (f64.const 1)
                                            (f64.const 1)
                                            (f64.const 0)
                                            (f64.const 0)
                                            (f64.const 0)
                                            (f64.const 1)
                                            (f64.const 0.82)
                                            (f64.const 1)
                                            (f64.const 0)
                                            (i32.const 88)
                                            (call $malloc)
                                            (local.set $_rpu_temp_mem_ptr)
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
                                            (return)
                                        )
                                    )
                                    (else
                                        (block
                                            local.get $id_59
                                            (f64.const 523.232)
                                            (f64.mul)
                                            (call $_rpu_vec1_fract_f64)
                                            local.set $d_60
                                            local.get $id_59
                                            (f64.const 1000)
                                            (f64.mul)
                                            (call $hash31)
                                            local.set $c_61_z
                                            local.set $c_61_y
                                            local.set $c_61_x

                                            local.get $d_60
                                            (f64.const 0.5)
                                            (f64.lt)
                                            (if
                                                (then
                                                    (block
                                                        (i64.const 1)
                                                        local.get $c_61_x
                                                        local.get $c_61_y
                                                        local.get $c_61_z
                                                        local.get $c_61_x
                                                        local.get $c_61_y
                                                        local.get $c_61_z
                                                        (call $_rpu_vec3_mul_vec3_f64)
                                                        (f64.const 0)
                                                        (f64.const 0)
                                                        (f64.const 0)
                                                        (f64.const 1)
                                                        (f64.const 1)
                                                        (f64.const 1)
                                                        (f64.const 0)
                                                        (i32.const 88)
                                                        (call $malloc)
                                                        (local.set $_rpu_temp_mem_ptr)
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
                                                        (return)
                                                    )
                                                )
                                                (else

                                                    local.get $d_60
                                                    (f64.const 0.8)
                                                    (f64.lt)
                                                    (if
                                                        (then
                                                            (block
                                                                (i64.const 1)
                                                                local.get $c_61_x
                                                                local.get $c_61_y
                                                                local.get $c_61_z
                                                                local.get $c_61_x
                                                                local.get $c_61_y
                                                                local.get $c_61_z
                                                                (call $_rpu_vec3_mul_vec3_f64)
                                                                (f64.const 0)
                                                                (f64.const 0)
                                                                (f64.const 0)
                                                                local.get $d_60
                                                                (f64.const 0.5)
                                                                local.get $d_60
                                                                (f64.mul)
                                                                (f64.const 1)
                                                                (f64.const 1.55)
                                                                (i32.const 88)
                                                                (call $malloc)
                                                                (local.set $_rpu_temp_mem_ptr)
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
                                                                (return)
                                                            )
                                                        )
                                                        (else
                                                            (block
                                                                (i64.const 1)
                                                                local.get $c_61_x
                                                                local.get $c_61_y
                                                                local.get $c_61_z
                                                                local.get $c_61_x
                                                                local.get $c_61_y
                                                                local.get $c_61_z
                                                                (f64.const 5)
                                                                (call $_rpu_vec3_mul_scalar_f64)
                                                                (f64.const 0.5)
                                                                (f64.const 0)
                                                                (f64.const 0)
                                                                (f64.const 0)
                                                                (i32.const 88)
                                                                (call $malloc)
                                                                (local.set $_rpu_temp_mem_ptr)
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
                                                                (return)
                                                            )
                                                        )
                                                    )
                                                )
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
        (i64.const 1)
        (f64.const 0.5)
        (f64.const 0.5)
        (f64.const 0.5)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0.5)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (i32.const 88)
        (call $malloc)
        (local.set $_rpu_temp_mem_ptr)
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
        (return)
    )

    ;; function 'GetRayDir'
    (func $GetRayDir (param $uv_62_x f64) (param $uv_62_y f64)(param $p_63_x f64) (param $p_63_y f64) (param $p_63_z f64)(param $l_64_x f64) (param $l_64_y f64) (param $l_64_z f64)(param $z_65 f64) (result f64 f64 f64)
        (local $f_66_x f64)
        (local $f_66_y f64)
        (local $f_66_z f64)
        (local $r_67_x f64)
        (local $r_67_y f64)
        (local $r_67_z f64)
        (local $u_68_x f64)
        (local $u_68_y f64)
        (local $u_68_z f64)
        (local $c_69_x f64)
        (local $c_69_y f64)
        (local $c_69_z f64)
        (local $i_70_x f64)
        (local $i_70_y f64)
        (local $i_70_z f64)
        local.get $l_64_x
        local.get $l_64_y
        local.get $l_64_z
        local.get $p_63_x
        local.get $p_63_y
        local.get $p_63_z
        (call $_rpu_vec3_sub_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $f_66_z
        local.set $f_66_y
        local.set $f_66_x
        (f64.const 0)
        (f64.const 1)
        (f64.const 0)
        local.get $f_66_x
        local.get $f_66_y
        local.get $f_66_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $r_67_z
        local.set $r_67_y
        local.set $r_67_x
        local.get $f_66_x
        local.get $f_66_y
        local.get $f_66_z
        local.get $r_67_x
        local.get $r_67_y
        local.get $r_67_z
        (call $_rpu_cross_product_f64)
        local.set $u_68_z
        local.set $u_68_y
        local.set $u_68_x
        local.get $f_66_x
        local.get $f_66_y
        local.get $f_66_z
        local.get $z_65
        (call $_rpu_vec3_mul_scalar_f64)
        local.set $c_69_z
        local.set $c_69_y
        local.set $c_69_x
        local.get $c_69_x
        local.get $c_69_y
        local.get $c_69_z
        local.get $uv_62_x
        local.get $r_67_x
        local.get $r_67_y
        local.get $r_67_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $uv_62_y
        local.get $u_68_x
        local.get $u_68_y
        local.get $u_68_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.set $i_70_z
        local.set $i_70_y
        local.set $i_70_x
        local.get $i_70_x
        local.get $i_70_y
        local.get $i_70_z
        (call $_rpu_normalize_vec3_f64)
        (return)
    )

    ;; function 'compNormal'
    (func $compNormal (param $p_71_x f64) (param $p_71_y f64) (param $p_71_z f64) (result f64 f64 f64)
        (local $e_72_x f64)
        (local $e_72_y f64)
        (local $_rpu_temp_f64 f64)
        (local $n_73_x f64)
        (local $n_73_y f64)
        (local $n_73_z f64)
        (f64.const 0.001)
        (f64.const 0)
        local.set $e_72_y
        local.set $e_72_x
        local.get $p_71_x
        local.get $p_71_y
        local.get $p_71_z
        (call $getDist)
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
        local.get $p_71_x
        local.get $p_71_y
        local.get $p_71_z
        local.get $e_72_x
        local.get $e_72_y
        local.get $e_72_y
        (call $_rpu_vec3_sub_vec3_f64)
        (call $getDist)
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
        local.get $p_71_x
        local.get $p_71_y
        local.get $p_71_z
        local.get $e_72_y
        local.get $e_72_x
        local.get $e_72_y
        (call $_rpu_vec3_sub_vec3_f64)
        (call $getDist)
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
        local.get $p_71_x
        local.get $p_71_y
        local.get $p_71_z
        local.get $e_72_y
        local.get $e_72_y
        local.get $e_72_x
        (call $_rpu_vec3_sub_vec3_f64)
        (call $getDist)
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
        (call $_rpu_scalar_sub_vec3_f64)
        local.set $n_73_z
        local.set $n_73_y
        local.set $n_73_x
        local.get $n_73_x
        local.get $n_73_y
        local.get $n_73_z
        (call $_rpu_normalize_vec3_f64)
        (return)
    )

    ;; function 'randomInUnitDisk'
    (func $randomInUnitDisk  (result f64 f64)
        (local $h_74_x f64)
        (local $h_74_y f64)
        (local $phi_75 f64)
        (local $r_76 f64)
        (call $_rpu_rand)
        (call $_rpu_rand)
        (f64.const 1)
        (f64.const 6.2831855)
        (call $_rpu_vec2_mul_vec2_f64)
        local.set $h_74_y
        local.set $h_74_x
        local.get $h_74_y
        local.set $phi_75
        local.get $h_74_x
        (call $_rpu_vec1_sqrt_f64)
        local.set $r_76
        local.get $r_76
        local.get $phi_75
        (call $_rpu_vec1_sin_f64)
        local.get $phi_75
        (call $_rpu_vec1_cos_f64)
        (call $_rpu_scalar_mul_vec2_f64)
        (return)
    )

    ;; function 'randomInUnitSphere'
    (func $randomInUnitSphere  (result f64 f64 f64)
        (local $h_77_x f64)
        (local $h_77_y f64)
        (local $h_77_z f64)
        (local $phi_78 f64)
        (local $r_79 f64)
        (call $_rpu_rand)
        (call $_rpu_rand)
        (call $_rpu_rand)
        (f64.const 2)
        (f64.const 6.2831855)
        (f64.const 1)
        (call $_rpu_vec3_mul_vec3_f64)
        (f64.const 1)
        (f64.const 0)
        (f64.const 0)
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $h_77_z
        local.set $h_77_y
        local.set $h_77_x
        local.get $h_77_y
        local.set $phi_78
        local.get $h_77_z
        (f64.const 1)
        (f64.const 3)
        (f64.div)
        (call $_rpu_vec1_pow_f64)
        local.set $r_79
        local.get $r_79
        (f64.const 1)
        local.get $h_77_x
        local.get $h_77_x
        (f64.mul)
        (f64.sub)
        (call $_rpu_vec1_sqrt_f64)
        local.get $phi_78
        (call $_rpu_vec1_sin_f64)
        local.get $phi_78
        (call $_rpu_vec1_cos_f64)
        (call $_rpu_scalar_mul_vec2_f64)
        local.get $h_77_x
        (call $_rpu_scalar_mul_vec3_f64)
        (return)
    )

    ;; function 'jitter'
    (func $jitter (param $d_80_x f64) (param $d_80_y f64) (param $d_80_z f64)(param $phi_81 f64)(param $sina_82 f64)(param $cosa_83 f64) (result f64 f64 f64)
        (local $w_84_x f64)
        (local $w_84_y f64)
        (local $w_84_z f64)
        (local $u_85_x f64)
        (local $u_85_y f64)
        (local $u_85_z f64)
        (local $v_86_x f64)
        (local $v_86_y f64)
        (local $v_86_z f64)
        local.get $d_80_x
        local.get $d_80_y
        local.get $d_80_z
        (call $_rpu_normalize_vec3_f64)
        local.set $w_84_z
        local.set $w_84_y
        local.set $w_84_x
        local.get $w_84_y
        local.get $w_84_z
        local.get $w_84_x
        local.get $w_84_x
        local.get $w_84_y
        local.get $w_84_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $u_85_z
        local.set $u_85_y
        local.set $u_85_x
        local.get $w_84_x
        local.get $w_84_y
        local.get $w_84_z
        local.get $u_85_x
        local.get $u_85_y
        local.get $u_85_z
        (call $_rpu_cross_product_f64)
        local.set $v_86_z
        local.set $v_86_y
        local.set $v_86_x
        local.get $u_85_x
        local.get $u_85_y
        local.get $u_85_z
        local.get $phi_81
        (call $_rpu_vec1_cos_f64)
        (call $_rpu_vec3_mul_scalar_f64)
        local.get $v_86_x
        local.get $v_86_y
        local.get $v_86_z
        local.get $phi_81
        (call $_rpu_vec1_sin_f64)
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $sina_82
        (call $_rpu_vec3_mul_scalar_f64)
        local.get $w_84_x
        local.get $w_84_y
        local.get $w_84_z
        local.get $cosa_83
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        (return)
    )

    ;; function 'createRay'
    (func $createRay (param $uv_87_x f64) (param $uv_87_y f64)(param $resolution_88_x f64) (param $resolution_88_y f64)(param $origin_89_x f64) (param $origin_89_y f64) (param $origin_89_z f64)(param $lookAt_90_x f64) (param $lookAt_90_y f64) (param $lookAt_90_z f64) (result i32)
        (local $_rpu_temp_mem_ptr i32)
        (local $_rpu_temp_f64 f64)
        (local $ray_91 i32)
        (local $aperture_92 f64)
        (local $focus_dist_93 f64)
        (local $fov_94 f64)
        (local $lens_radius_95 f64)
        (local $theta_96 f64)
        (local $half_height_97 f64)
        (local $half_width_98 f64)
        (local $w_99_x f64)
        (local $w_99_y f64)
        (local $w_99_z f64)
        (local $u_100_x f64)
        (local $u_100_y f64)
        (local $u_100_z f64)
        (local $v_101_x f64)
        (local $v_101_y f64)
        (local $v_101_z f64)
        (local $lower_left_corner_102_x f64)
        (local $lower_left_corner_102_y f64)
        (local $lower_left_corner_102_z f64)
        (local $horizontal_103_x f64)
        (local $horizontal_103_y f64)
        (local $horizontal_103_z f64)
        (local $vertical_104_x f64)
        (local $vertical_104_y f64)
        (local $vertical_104_z f64)
        (local $unit_105_x f64)
        (local $unit_105_y f64)
        (local $offset_106_x f64)
        (local $offset_106_y f64)
        (local $offset_106_z f64)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
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
        (local.set $ray_91)
        (f64.const 0.1)
        local.set $aperture_92
        (f64.const 10)
        local.set $focus_dist_93
        (f64.const 20)
        local.set $fov_94
        local.get $aperture_92
        (f64.const 2)
        (f64.div)
        local.set $lens_radius_95
        local.get $fov_94
        (call $_rpu_vec1_radians_f64)
        local.set $theta_96
        local.get $theta_96
        (f64.const 2)
        (f64.div)
        (call $_rpu_vec1_tan_f64)
        local.set $half_height_97
        local.get $resolution_88_x
        local.get $resolution_88_y
        (f64.div)
        local.get $half_height_97
        (f64.mul)
        local.set $half_width_98
        local.get $origin_89_x
        local.get $origin_89_y
        local.get $origin_89_z
        local.get $lookAt_90_x
        local.get $lookAt_90_y
        local.get $lookAt_90_z
        (call $_rpu_vec3_sub_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $w_99_z
        local.set $w_99_y
        local.set $w_99_x
        (f64.const 0)
        (f64.const 1)
        (f64.const 0)
        local.get $w_99_x
        local.get $w_99_y
        local.get $w_99_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $u_100_z
        local.set $u_100_y
        local.set $u_100_x
        local.get $w_99_x
        local.get $w_99_y
        local.get $w_99_z
        local.get $u_100_x
        local.get $u_100_y
        local.get $u_100_z
        (call $_rpu_cross_product_f64)
        local.set $v_101_z
        local.set $v_101_y
        local.set $v_101_x
        local.get $origin_89_x
        local.get $origin_89_y
        local.get $origin_89_z
        local.get $half_width_98
        local.get $focus_dist_93
        (f64.mul)
        local.get $u_100_x
        local.get $u_100_y
        local.get $u_100_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_sub_vec3_f64)
        local.get $half_height_97
        local.get $focus_dist_93
        (f64.mul)
        local.get $v_101_x
        local.get $v_101_y
        local.get $v_101_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_sub_vec3_f64)
        local.get $focus_dist_93
        local.get $w_99_x
        local.get $w_99_y
        local.get $w_99_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $lower_left_corner_102_z
        local.set $lower_left_corner_102_y
        local.set $lower_left_corner_102_x
        (f64.const 2)
        local.get $half_width_98
        (f64.mul)
        local.get $focus_dist_93
        (f64.mul)
        local.get $u_100_x
        local.get $u_100_y
        local.get $u_100_z
        (call $_rpu_scalar_mul_vec3_f64)
        local.set $horizontal_103_z
        local.set $horizontal_103_y
        local.set $horizontal_103_x
        (f64.const 2)
        local.get $half_height_97
        (f64.mul)
        local.get $focus_dist_93
        (f64.mul)
        local.get $v_101_x
        local.get $v_101_y
        local.get $v_101_z
        (call $_rpu_scalar_mul_vec3_f64)
        local.set $vertical_104_z
        local.set $vertical_104_y
        local.set $vertical_104_x
        local.get $lens_radius_95
        (call $randomInUnitDisk)
        (call $_rpu_scalar_mul_vec2_f64)
        local.set $unit_105_y
        local.set $unit_105_x
        local.get $u_100_x
        local.get $u_100_y
        local.get $u_100_z
        local.get $unit_105_x
        (call $_rpu_vec3_mul_scalar_f64)
        local.get $v_101_x
        local.get $v_101_y
        local.get $v_101_z
        local.get $unit_105_y
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.set $offset_106_z
        local.set $offset_106_y
        local.set $offset_106_x
        local.get $origin_89_x
        local.get $origin_89_y
        local.get $origin_89_z
        local.get $offset_106_x
        local.get $offset_106_y
        local.get $offset_106_z
        (call $_rpu_vec3_add_vec3_f64)
        (local.set $_rpu_temp_f64)
        (local.get $ray_91)
        (i32.const 16)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $ray_91)
        (i32.const 8)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $ray_91)
        (i32.const 0)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        local.get $lower_left_corner_102_x
        local.get $lower_left_corner_102_y
        local.get $lower_left_corner_102_z
        local.get $uv_87_x
        local.get $horizontal_103_x
        local.get $horizontal_103_y
        local.get $horizontal_103_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $uv_87_y
        local.get $vertical_104_x
        local.get $vertical_104_y
        local.get $vertical_104_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $origin_89_x
        local.get $origin_89_y
        local.get $origin_89_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.get $offset_106_x
        local.get $offset_106_y
        local.get $offset_106_z
        (call $_rpu_vec3_sub_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        (local.set $_rpu_temp_f64)
        (local.get $ray_91)
        (i32.const 40)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $ray_91)
        (i32.const 32)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $ray_91)
        (i32.const 24)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.get $ray_91)
        (return)
    )

    ;; function 'raymarch'
    (func $raymarch (param $ray_107 i32)(param $max_dist_108 f64) (result f64 f64)
        (local $t_109 f64)
        (local $p_110_x f64)
        (local $p_110_y f64)
        (local $p_110_z f64)
        (local $rc_111_x f64)
        (local $rc_111_y f64)
        (local $d_112 f64)
        (f64.const 0.1)
        local.set $t_109

        (block
            (loop
                local.get $t_109
                local.get $max_dist_108
                (f64.lt)
                (i32.eqz)
                (br_if 1)
                (block
                    (local.get $ray_107)
                    (i32.const 0)
                    (i32.add)
                    (f64.load)
                    (local.get $ray_107)
                    (i32.const 8)
                    (i32.add)
                    (f64.load)
                    (local.get $ray_107)
                    (i32.const 16)
                    (i32.add)
                    (f64.load)
                    (local.get $ray_107)
                    (i32.const 24)
                    (i32.add)
                    (f64.load)
                    (local.get $ray_107)
                    (i32.const 32)
                    (i32.add)
                    (f64.load)
                    (local.get $ray_107)
                    (i32.const 40)
                    (i32.add)
                    (f64.load)
                    local.get $t_109
                    (call $_rpu_vec3_mul_scalar_f64)
                    (call $_rpu_vec3_add_vec3_f64)
                    local.set $p_110_z
                    local.set $p_110_y
                    local.set $p_110_x
                    local.get $p_110_x
                    local.get $p_110_y
                    local.get $p_110_z
                    (call $getDist)
                    local.set $rc_111_y
                    local.set $rc_111_x
                    local.get $rc_111_x
                    (call $_rpu_vec1_abs_f64)
                    local.set $d_112

                    local.get $d_112
                    (f64.const 0.01)
                    (f64.lt)
                    (if
                        (then
                            (block
                                local.get $t_109
                                local.get $rc_111_y
                                (return)
                            )
                        )
                    )
                    local.get $d_112
                    local.get $t_109
                    f64.add
                    local.set $t_109
                )
                (br 0)
            )
        )
        local.get $t_109
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        (return)
    )

    ;; function 'ggx'
    (func $ggx (param $N_113_x f64) (param $N_113_y f64) (param $N_113_z f64)(param $V_114_x f64) (param $V_114_y f64) (param $V_114_z f64)(param $L_115_x f64) (param $L_115_y f64) (param $L_115_z f64)(param $roughness_116 f64)(param $F0_117 f64) (result f64)
        (local $H_118_x f64)
        (local $H_118_y f64)
        (local $H_118_z f64)
        (local $dotLH_119 f64)
        (local $dotNH_120 f64)
        (local $dotNL_121 f64)
        (local $dotNV_122 f64)
        (local $alpha_123 f64)
        (local $alphaSqr_124 f64)
        (local $denom_125 f64)
        (local $D_126 f64)
        (local $F_a_127 f64)
        (local $F_b_128 f64)
        (local $F_129 f64)
        (local $k_130 f64)
        (local $G_131 f64)
        local.get $V_114_x
        local.get $V_114_y
        local.get $V_114_z
        local.get $L_115_x
        local.get $L_115_y
        local.get $L_115_z
        (call $_rpu_vec3_add_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $H_118_z
        local.set $H_118_y
        local.set $H_118_x
        local.get $L_115_x
        local.get $L_115_y
        local.get $L_115_z
        local.get $H_118_x
        local.get $H_118_y
        local.get $H_118_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.const 0)
        (call $_rpu_vec1_max_f64)
        local.set $dotLH_119
        local.get $N_113_x
        local.get $N_113_y
        local.get $N_113_z
        local.get $H_118_x
        local.get $H_118_y
        local.get $H_118_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.const 0)
        (call $_rpu_vec1_max_f64)
        local.set $dotNH_120
        local.get $N_113_x
        local.get $N_113_y
        local.get $N_113_z
        local.get $L_115_x
        local.get $L_115_y
        local.get $L_115_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.const 0)
        (call $_rpu_vec1_max_f64)
        local.set $dotNL_121
        local.get $N_113_x
        local.get $N_113_y
        local.get $N_113_z
        local.get $V_114_x
        local.get $V_114_y
        local.get $V_114_z
        (call $_rpu_dot_product_vec3_f64)
        (f64.const 0)
        (call $_rpu_vec1_max_f64)
        local.set $dotNV_122
        local.get $roughness_116
        local.get $roughness_116
        (f64.mul)
        (f64.const 0.0001)
        (f64.add)
        local.set $alpha_123
        local.get $alpha_123
        local.get $alpha_123
        (f64.mul)
        local.set $alphaSqr_124
        local.get $dotNH_120
        local.get $dotNH_120
        (f64.mul)
        local.get $alphaSqr_124
        (f64.const 1)
        (f64.sub)
        (f64.mul)
        (f64.const 1)
        (f64.add)
        local.set $denom_125
        local.get $alphaSqr_124
        local.get $denom_125
        local.get $denom_125
        (f64.mul)
        (f64.div)
        local.set $D_126
        (f64.const 1)
        local.set $F_a_127
        (f64.const 1)
        local.get $dotLH_119
        (f64.sub)
        (f64.const 5)
        (call $_rpu_vec1_pow_f64)
        local.set $F_b_128
        local.get $F_b_128
        local.get $F_a_127
        local.get $F0_117
        (call $_rpu_mix_vec1_f64)
        local.set $F_129
        local.get $alpha_123
        (f64.const 2)
        local.get $roughness_116
        (f64.mul)
        (f64.add)
        (f64.const 1)
        (f64.add)
        (f64.const 8)
        (f64.div)
        local.set $k_130
        local.get $dotNL_121
        local.get $dotNL_121
        (f64.const 1)
        local.get $k_130
        (call $_rpu_mix_vec1_f64)
        local.get $dotNV_122
        (f64.const 1)
        local.get $k_130
        (call $_rpu_mix_vec1_f64)
        (f64.mul)
        (f64.div)
        local.set $G_131
        (f64.const 0)
        (f64.const 10)
        local.get $D_126
        local.get $F_129
        (f64.mul)
        local.get $G_131
        (f64.mul)
        (f64.const 4)
        (f64.div)
        (call $_rpu_vec1_min_f64)
        (call $_rpu_vec1_max_f64)
        (return)
    )

    ;; function 'angleToDir'
    (func $angleToDir (param $n_132_x f64) (param $n_132_y f64) (param $n_132_z f64)(param $theta_133 f64)(param $phi_134 f64) (result f64 f64 f64)
        (local $sinPhi_135 f64)
        (local $cosPhi_136 f64)
        (local $w_137_x f64)
        (local $w_137_y f64)
        (local $w_137_z f64)
        (local $u_138_x f64)
        (local $u_138_y f64)
        (local $u_138_z f64)
        (local $v_139_x f64)
        (local $v_139_y f64)
        (local $v_139_z f64)
        local.get $phi_134
        (call $_rpu_vec1_sin_f64)
        local.set $sinPhi_135
        local.get $phi_134
        (call $_rpu_vec1_cos_f64)
        local.set $cosPhi_136
        local.get $n_132_x
        local.get $n_132_y
        local.get $n_132_z
        (call $_rpu_normalize_vec3_f64)
        local.set $w_137_z
        local.set $w_137_y
        local.set $w_137_x
        local.get $w_137_y
        local.get $w_137_z
        local.get $w_137_x
        local.get $w_137_x
        local.get $w_137_y
        local.get $w_137_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $u_138_z
        local.set $u_138_y
        local.set $u_138_x
        local.get $w_137_x
        local.get $w_137_y
        local.get $w_137_z
        local.get $u_138_x
        local.get $u_138_y
        local.get $u_138_z
        (call $_rpu_cross_product_f64)
        local.set $v_139_z
        local.set $v_139_y
        local.set $v_139_x
        local.get $u_138_x
        local.get $u_138_y
        local.get $u_138_z
        local.get $theta_133
        (call $_rpu_vec1_cos_f64)
        (call $_rpu_vec3_mul_scalar_f64)
        local.get $v_139_x
        local.get $v_139_y
        local.get $v_139_z
        local.get $theta_133
        (call $_rpu_vec1_sin_f64)
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $sinPhi_135
        (call $_rpu_vec3_mul_scalar_f64)
        local.get $w_137_x
        local.get $w_137_y
        local.get $w_137_z
        local.get $cosPhi_136
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        (return)
    )

    ;; function 'radiance'
    (func $radiance (param $r_140 i32) (result f64 f64 f64)
        (local $acc_141_x f64)
        (local $acc_141_y f64)
        (local $acc_141_z f64)
        (local $mask_142_x f64)
        (local $mask_142_y f64)
        (local $mask_142_z f64)
        (local $depth_143 i64)
        (local $hit_144_x f64)
        (local $hit_144_y f64)
        (local $t_145 f64)
        (local $_rpu_temp_f64 f64)
        (local $material_146 i32)
        (local $x_147_x f64)
        (local $x_147_y f64)
        (local $x_147_z f64)
        (local $n_148_x f64)
        (local $n_148_y f64)
        (local $n_148_z f64)
        (local $nl_149_x f64)
        (local $nl_149_y f64)
        (local $nl_149_z f64)
        (local $r2_150 f64)
        (local $d_151_x f64)
        (local $d_151_y f64)
        (local $d_151_z f64)
        (local $e_152_x f64)
        (local $e_152_y f64)
        (local $e_152_z f64)
        (local $_rpu_temp_mem_ptr i32)
        (local $E_153 f64)
        (local $roughness_154 f64)
        (local $alpha_155 f64)
        (local $metallic_156 f64)
        (local $reflectance_157 f64)
        (local $color_158_x f64)
        (local $color_158_y f64)
        (local $color_158_z f64)
        (local $brdf_159_x f64)
        (local $brdf_159_y f64)
        (local $brdf_159_z f64)
        (local $xsi_1_160 f64)
        (local $xsi_2_161 f64)
        (local $phi_162 f64)
        (local $theta_163 f64)
        (local $direction_164_x f64)
        (local $direction_164_y f64)
        (local $direction_164_z f64)
        (local $a_165 f64)
        (local $ddn_166 f64)
        (local $nc_167 f64)
        (local $nt_168 f64)
        (local $_rpu_ternary_3 f64)
        (local $nnt_169 f64)
        (local $cos2t_170 f64)
        (local $tdir_171_x f64)
        (local $tdir_171_y f64)
        (local $tdir_171_z f64)
        (local $R0_172 f64)
        (local $_rpu_ternary_4 f64)
        (local $c_173 f64)
        (local $Re_174 f64)
        (local $P_175 f64)
        (local $RP_176 f64)
        (local $TP_177 f64)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $acc_141_z
        local.set $acc_141_y
        local.set $acc_141_x
        (f64.const 1)
        (f64.const 1)
        (f64.const 1)
        local.set $mask_142_z
        local.set $mask_142_y
        local.set $mask_142_x

        (i64.const 0)
        local.set $depth_143
        (block
            (loop
                local.get $depth_143
                (i64.const 8)
                (i64.lt_s)
                (i32.eqz)
                (br_if 1)
                (block
                    (local.get $r_140)
                    (f64.const 20)
                    (call $raymarch)
                    local.set $hit_144_y
                    local.set $hit_144_x

                    local.get $hit_144_y
                    (f64.const 0)
                    (f64.lt)
                    (if
                        (then
                            (block
                                (f64.const 0.5)
                                (local.get $r_140)
                                (i32.const 32)
                                (i32.add)
                                (f64.load)
                                (f64.mul)
                                (f64.const 0.5)
                                (f64.add)
                                local.set $t_145
                                local.get $mask_142_x
                                local.get $mask_142_y
                                local.get $mask_142_z
                                (f64.const 1)
                                (f64.const 1)
                                (f64.const 1)
                                (f64.const 0.5)
                                (f64.const 0.7)
                                (f64.const 1)
                                local.get $t_145
                                (call $_rpu_mix_vec3_f64)
                                (call $_rpu_vec3_mul_vec3_f64)
                                local.set $_rpu_temp_f64
                                local.get $acc_141_z
                                local.get $_rpu_temp_f64
                                f64.add
                                local.set $acc_141_z
                                local.set $_rpu_temp_f64
                                local.get $acc_141_y
                                local.get $_rpu_temp_f64
                                f64.add
                                local.set $acc_141_y
                                local.set $_rpu_temp_f64
                                local.get $acc_141_x
                                local.get $_rpu_temp_f64
                                f64.add
                                local.set $acc_141_x
                                (br 4)
                            )
                        )
                    )

                    local.get $depth_143
                    (i64.const 1)
                    (i64.gt_s)
                    (if
                        (then
                            (block

                                (call $_rpu_rand)
                                local.get $mask_142_x
                                local.get $mask_142_y
                                (call $_rpu_vec1_max_f64)
                                local.get $mask_142_z
                                (call $_rpu_vec1_max_f64)
                                (f64.gt)
                                (if
                                    (then
                                        (block
                                            local.get $acc_141_x
                                            local.get $acc_141_y
                                            local.get $acc_141_z
                                            (return)
                                        )
                                    )
                                    (else
                                        (block
                                            local.get $mask_142_x
                                            local.get $mask_142_y
                                            local.get $mask_142_z
                                            (f64.const 1)
                                            local.get $mask_142_x
                                            local.get $mask_142_y
                                            (call $_rpu_vec1_max_f64)
                                            local.get $mask_142_z
                                            (call $_rpu_vec1_max_f64)
                                            (f64.div)
                                            (call $_rpu_vec3_mul_scalar_f64)
                                            local.set $mask_142_z
                                            local.set $mask_142_y
                                            local.set $mask_142_x
                                        )
                                    )
                                )
                            )
                        )
                    )
                    local.get $hit_144_y
                    (call $getMaterial)
                    (local.set $material_146)
                    (local.get $r_140)
                    (i32.const 0)
                    (i32.add)
                    (f64.load)
                    (local.get $r_140)
                    (i32.const 8)
                    (i32.add)
                    (f64.load)
                    (local.get $r_140)
                    (i32.const 16)
                    (i32.add)
                    (f64.load)
                    local.get $hit_144_x
                    (local.get $r_140)
                    (i32.const 24)
                    (i32.add)
                    (f64.load)
                    (local.get $r_140)
                    (i32.const 32)
                    (i32.add)
                    (f64.load)
                    (local.get $r_140)
                    (i32.const 40)
                    (i32.add)
                    (f64.load)
                    (call $_rpu_scalar_mul_vec3_f64)
                    (call $_rpu_vec3_add_vec3_f64)
                    local.set $x_147_z
                    local.set $x_147_y
                    local.set $x_147_x
                    local.get $x_147_x
                    local.get $x_147_y
                    local.get $x_147_z
                    (call $compNormal)
                    local.set $n_148_z
                    local.set $n_148_y
                    local.set $n_148_x
                    local.get $n_148_x
                    local.get $n_148_y
                    local.get $n_148_z
                    local.get $n_148_x
                    local.get $n_148_y
                    local.get $n_148_z
                    (local.get $r_140)
                    (i32.const 24)
                    (i32.add)
                    (f64.load)
                    (local.get $r_140)
                    (i32.const 32)
                    (i32.add)
                    (f64.load)
                    (local.get $r_140)
                    (i32.const 40)
                    (i32.add)
                    (f64.load)
                    (call $_rpu_dot_product_vec3_f64)
                    (call $_rpu_vec1_neg_f64)
                    (call $_rpu_vec1_sign_f64)
                    (call $_rpu_vec3_mul_scalar_f64)
                    local.set $nl_149_z
                    local.set $nl_149_y
                    local.set $nl_149_x

                    (local.get $material_146)
                    (i32.const 0)
                    (i32.add)
                    (i64.load)
                    (i64.const 0)
                    (i64.eq)
                    (if
                        (then
                            (block
                                (call $_rpu_rand)
                                local.set $r2_150
                                local.get $nl_149_x
                                local.get $nl_149_y
                                local.get $nl_149_z
                                (f64.const 2)
                                (f64.const 3.1415927)
                                (f64.mul)
                                (call $_rpu_rand)
                                (f64.mul)
                                local.get $r2_150
                                (call $_rpu_vec1_sqrt_f64)
                                (f64.const 1)
                                local.get $r2_150
                                (f64.sub)
                                (call $_rpu_vec1_sqrt_f64)
                                (call $jitter)
                                local.set $d_151_z
                                local.set $d_151_y
                                local.set $d_151_x
                                (f64.const 0)
                                (f64.const 0)
                                (f64.const 0)
                                local.set $e_152_z
                                local.set $e_152_y
                                local.set $e_152_x
                                local.get $mask_142_x
                                local.get $mask_142_y
                                local.get $mask_142_z
                                (local.get $material_146)
                                (i32.const 32)
                                (i32.add)
                                (f64.load)
                                (local.get $material_146)
                                (i32.const 40)
                                (i32.add)
                                (f64.load)
                                (local.get $material_146)
                                (i32.const 48)
                                (i32.add)
                                (f64.load)
                                (call $_rpu_vec3_mul_vec3_f64)
                                local.get $mask_142_x
                                local.get $mask_142_y
                                local.get $mask_142_z
                                (local.get $material_146)
                                (i32.const 8)
                                (i32.add)
                                (f64.load)
                                (local.get $material_146)
                                (i32.const 16)
                                (i32.add)
                                (f64.load)
                                (local.get $material_146)
                                (i32.const 24)
                                (i32.add)
                                (f64.load)
                                (call $_rpu_vec3_mul_vec3_f64)
                                local.get $e_152_x
                                local.get $e_152_y
                                local.get $e_152_z
                                (call $_rpu_vec3_mul_vec3_f64)
                                (call $_rpu_vec3_add_vec3_f64)
                                local.set $_rpu_temp_f64
                                local.get $acc_141_z
                                local.get $_rpu_temp_f64
                                f64.add
                                local.set $acc_141_z
                                local.set $_rpu_temp_f64
                                local.get $acc_141_y
                                local.get $_rpu_temp_f64
                                f64.add
                                local.set $acc_141_y
                                local.set $_rpu_temp_f64
                                local.get $acc_141_x
                                local.get $_rpu_temp_f64
                                f64.add
                                local.set $acc_141_x
                                (local.get $material_146)
                                (i32.const 8)
                                (i32.add)
                                (f64.load)
                                (local.get $material_146)
                                (i32.const 16)
                                (i32.add)
                                (f64.load)
                                (local.get $material_146)
                                (i32.const 24)
                                (i32.add)
                                (f64.load)
                                local.set $_rpu_temp_f64
                                local.get $mask_142_z
                                local.get $_rpu_temp_f64
                                f64.mul
                                local.set $mask_142_z
                                local.set $_rpu_temp_f64
                                local.get $mask_142_y
                                local.get $_rpu_temp_f64
                                f64.mul
                                local.set $mask_142_y
                                local.set $_rpu_temp_f64
                                local.get $mask_142_x
                                local.get $_rpu_temp_f64
                                f64.mul
                                local.set $mask_142_x
                                local.get $x_147_x
                                local.get $x_147_y
                                local.get $x_147_z
                                local.get $d_151_x
                                local.get $d_151_y
                                local.get $d_151_z
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
                                (local.set $r_140)
                            )
                        )
                        (else

                            (local.get $material_146)
                            (i32.const 0)
                            (i32.add)
                            (i64.load)
                            (i64.const 1)
                            (i64.eq)
                            (if
                                (then
                                    (block
                                        (f64.const 1)
                                        local.set $E_153
                                        (f64.const 1)
                                        (local.get $material_146)
                                        (i32.const 56)
                                        (i32.add)
                                        (f64.load)
                                        (local.get $material_146)
                                        (i32.const 56)
                                        (i32.add)
                                        (f64.load)
                                        (f64.mul)
                                        (f64.sub)
                                        local.set $roughness_154
                                        local.get $roughness_154
                                        local.get $roughness_154
                                        (f64.mul)
                                        local.set $alpha_155
                                        (local.get $material_146)
                                        (i32.const 64)
                                        (i32.add)
                                        (f64.load)
                                        local.set $metallic_156
                                        (local.get $material_146)
                                        (i32.const 72)
                                        (i32.add)
                                        (f64.load)
                                        local.set $reflectance_157
                                        (local.get $material_146)
                                        (i32.const 8)
                                        (i32.add)
                                        (f64.load)
                                        (local.get $material_146)
                                        (i32.const 16)
                                        (i32.add)
                                        (f64.load)
                                        (local.get $material_146)
                                        (i32.const 24)
                                        (i32.add)
                                        (f64.load)
                                        local.set $color_158_z
                                        local.set $color_158_y
                                        local.set $color_158_x
                                        (f64.const 0)
                                        (f64.const 0)
                                        (f64.const 0)
                                        local.set $brdf_159_z
                                        local.set $brdf_159_y
                                        local.set $brdf_159_x
                                        (call $_rpu_rand)
                                        local.set $xsi_1_160
                                        (call $_rpu_rand)
                                        local.set $xsi_2_161
                                        local.get $alpha_155
                                        local.get $xsi_1_160
                                        (call $_rpu_vec1_sqrt_f64)
                                        (f64.mul)
                                        (f64.const 1)
                                        local.get $xsi_1_160
                                        (f64.sub)
                                        (call $_rpu_vec1_sqrt_f64)
                                        (f64.div)
                                        (call $_rpu_vec1_atan_f64)
                                        local.set $phi_162
                                        (f64.const 2)
                                        (f64.const 3.1415927)
                                        (f64.mul)
                                        local.get $xsi_2_161
                                        (f64.mul)
                                        local.set $theta_163
                                        local.get $nl_149_x
                                        local.get $nl_149_y
                                        local.get $nl_149_z
                                        local.get $theta_163
                                        local.get $phi_162
                                        (call $angleToDir)
                                        local.set $direction_164_z
                                        local.set $direction_164_y
                                        local.set $direction_164_x
                                        local.get $x_147_x
                                        local.get $x_147_y
                                        local.get $x_147_z
                                        local.get $direction_164_x
                                        local.get $direction_164_y
                                        local.get $direction_164_z
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
                                        (local.set $r_140)
                                        local.get $mask_142_x
                                        local.get $mask_142_y
                                        local.get $mask_142_z
                                        (local.get $material_146)
                                        (i32.const 32)
                                        (i32.add)
                                        (f64.load)
                                        (local.get $material_146)
                                        (i32.const 40)
                                        (i32.add)
                                        (f64.load)
                                        (local.get $material_146)
                                        (i32.const 48)
                                        (i32.add)
                                        (f64.load)
                                        (call $_rpu_vec3_mul_vec3_f64)
                                        local.get $E_153
                                        (call $_rpu_vec3_mul_scalar_f64)
                                        local.get $mask_142_x
                                        local.get $mask_142_y
                                        local.get $mask_142_z
                                        local.get $color_158_x
                                        local.get $color_158_y
                                        local.get $color_158_z
                                        (call $_rpu_vec3_mul_vec3_f64)
                                        local.get $brdf_159_x
                                        local.get $brdf_159_y
                                        local.get $brdf_159_z
                                        (call $_rpu_vec3_mul_vec3_f64)
                                        (call $_rpu_vec3_add_vec3_f64)
                                        local.set $_rpu_temp_f64
                                        local.get $acc_141_z
                                        local.get $_rpu_temp_f64
                                        f64.add
                                        local.set $acc_141_z
                                        local.set $_rpu_temp_f64
                                        local.get $acc_141_y
                                        local.get $_rpu_temp_f64
                                        f64.add
                                        local.set $acc_141_y
                                        local.set $_rpu_temp_f64
                                        local.get $acc_141_x
                                        local.get $_rpu_temp_f64
                                        f64.add
                                        local.set $acc_141_x
                                        local.get $color_158_x
                                        local.get $color_158_y
                                        local.get $color_158_z
                                        local.set $_rpu_temp_f64
                                        local.get $mask_142_z
                                        local.get $_rpu_temp_f64
                                        f64.mul
                                        local.set $mask_142_z
                                        local.set $_rpu_temp_f64
                                        local.get $mask_142_y
                                        local.get $_rpu_temp_f64
                                        f64.mul
                                        local.set $mask_142_y
                                        local.set $_rpu_temp_f64
                                        local.get $mask_142_x
                                        local.get $_rpu_temp_f64
                                        f64.mul
                                        local.set $mask_142_x
                                    )
                                )
                                (else

                                    (local.get $material_146)
                                    (i32.const 0)
                                    (i32.add)
                                    (i64.load)
                                    (i64.const 2)
                                    (i64.eq)
                                    (if
                                        (then
                                            (block
                                                local.get $n_148_x
                                                local.get $n_148_y
                                                local.get $n_148_z
                                                (local.get $r_140)
                                                (i32.const 24)
                                                (i32.add)
                                                (f64.load)
                                                (local.get $r_140)
                                                (i32.const 32)
                                                (i32.add)
                                                (f64.load)
                                                (local.get $r_140)
                                                (i32.const 40)
                                                (i32.add)
                                                (f64.load)
                                                (call $_rpu_dot_product_vec3_f64)
                                                local.set $a_165
                                                local.get $a_165
                                                (call $_rpu_vec1_abs_f64)
                                                local.set $ddn_166
                                                (f64.const 1)
                                                local.set $nc_167
                                                (local.get $material_146)
                                                (i32.const 80)
                                                (i32.add)
                                                (f64.load)
                                                local.set $nt_168
                                                local.get $nc_167
                                                local.get $nt_168
                                                (f64.div)
                                                local.get $nt_168
                                                local.get $nc_167
                                                (f64.div)

                                                local.get $a_165
                                                (f64.const 0)
                                                (f64.gt)
                                                (if
                                                    (then
                                                        (f64.const 1)
                                                        (local.set $_rpu_ternary_3)
                                                    )
                                                    (else
                                                        (f64.const 0)
                                                        (local.set $_rpu_ternary_3)
                                                    )
                                                )
                                                (local.get $_rpu_ternary_3)
                                                (call $_rpu_mix_vec1_f64)
                                                local.set $nnt_169
                                                (f64.const 1)
                                                local.get $nnt_169
                                                local.get $nnt_169
                                                (f64.mul)
                                                (f64.const 1)
                                                local.get $ddn_166
                                                local.get $ddn_166
                                                (f64.mul)
                                                (f64.sub)
                                                (f64.mul)
                                                (f64.sub)
                                                local.set $cos2t_170
                                                local.get $x_147_x
                                                local.get $x_147_y
                                                local.get $x_147_z
                                                (local.get $r_140)
                                                (i32.const 24)
                                                (i32.add)
                                                (f64.load)
                                                (local.get $r_140)
                                                (i32.const 32)
                                                (i32.add)
                                                (f64.load)
                                                (local.get $r_140)
                                                (i32.const 40)
                                                (i32.add)
                                                (f64.load)
                                                local.get $n_148_x
                                                local.get $n_148_y
                                                local.get $n_148_z
                                                (call $reflect)
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
                                                (local.set $r_140)

                                                local.get $cos2t_170
                                                (f64.const 0)
                                                (f64.gt)
                                                (if
                                                    (then
                                                        (block
                                                            (local.get $r_140)
                                                            (i32.const 24)
                                                            (i32.add)
                                                            (f64.load)
                                                            (local.get $r_140)
                                                            (i32.const 32)
                                                            (i32.add)
                                                            (f64.load)
                                                            (local.get $r_140)
                                                            (i32.const 40)
                                                            (i32.add)
                                                            (f64.load)
                                                            local.get $nnt_169
                                                            (call $_rpu_vec3_mul_scalar_f64)
                                                            local.get $a_165
                                                            (call $_rpu_vec1_sign_f64)
                                                            local.get $n_148_x
                                                            local.get $n_148_y
                                                            local.get $n_148_z
                                                            (call $_rpu_scalar_mul_vec3_f64)
                                                            local.get $ddn_166
                                                            local.get $nnt_169
                                                            (f64.mul)
                                                            local.get $cos2t_170
                                                            (call $_rpu_vec1_sqrt_f64)
                                                            (f64.add)
                                                            (call $_rpu_vec3_mul_scalar_f64)
                                                            (call $_rpu_vec3_add_vec3_f64)
                                                            (call $_rpu_normalize_vec3_f64)
                                                            local.set $tdir_171_z
                                                            local.set $tdir_171_y
                                                            local.set $tdir_171_x
                                                            local.get $nt_168
                                                            local.get $nc_167
                                                            (f64.sub)
                                                            local.get $nt_168
                                                            local.get $nc_167
                                                            (f64.sub)
                                                            (f64.mul)
                                                            local.get $nt_168
                                                            local.get $nc_167
                                                            (f64.add)
                                                            local.get $nt_168
                                                            local.get $nc_167
                                                            (f64.add)
                                                            (f64.mul)
                                                            (f64.div)
                                                            local.set $R0_172
                                                            (f64.const 1)
                                                            local.get $ddn_166
                                                            local.get $tdir_171_x
                                                            local.get $tdir_171_y
                                                            local.get $tdir_171_z
                                                            local.get $n_148_x
                                                            local.get $n_148_y
                                                            local.get $n_148_z
                                                            (call $_rpu_dot_product_vec3_f64)

                                                            local.get $a_165
                                                            (f64.const 0)
                                                            (f64.gt)
                                                            (if
                                                                (then
                                                                    (f64.const 1)
                                                                    (local.set $_rpu_ternary_4)
                                                                )
                                                                (else
                                                                    (f64.const 0)
                                                                    (local.set $_rpu_ternary_4)
                                                                )
                                                            )
                                                            (local.get $_rpu_ternary_4)
                                                            (call $_rpu_mix_vec1_f64)
                                                            (f64.sub)
                                                            local.set $c_173
                                                            local.get $R0_172
                                                            (f64.const 1)
                                                            local.get $R0_172
                                                            (f64.sub)
                                                            local.get $c_173
                                                            (f64.mul)
                                                            local.get $c_173
                                                            (f64.mul)
                                                            local.get $c_173
                                                            (f64.mul)
                                                            local.get $c_173
                                                            (f64.mul)
                                                            local.get $c_173
                                                            (f64.mul)
                                                            (f64.add)
                                                            local.set $Re_174
                                                            (f64.const 0.25)
                                                            (f64.const 0.5)
                                                            local.get $Re_174
                                                            (f64.mul)
                                                            (f64.add)
                                                            local.set $P_175
                                                            local.get $Re_174
                                                            local.get $P_175
                                                            (f64.div)
                                                            local.set $RP_176
                                                            (f64.const 1)
                                                            local.get $Re_174
                                                            (f64.sub)
                                                            (f64.const 1)
                                                            local.get $P_175
                                                            (f64.sub)
                                                            (f64.div)
                                                            local.set $TP_177

                                                            (call $_rpu_rand)
                                                            local.get $P_175
                                                            (f64.lt)
                                                            (if
                                                                (then
                                                                    local.get $RP_176
                                                                    local.get $RP_176
                                                                    local.get $RP_176
                                                                    local.set $_rpu_temp_f64
                                                                    local.get $mask_142_z
                                                                    local.get $_rpu_temp_f64
                                                                    f64.mul
                                                                    local.set $mask_142_z
                                                                    local.set $_rpu_temp_f64
                                                                    local.get $mask_142_y
                                                                    local.get $_rpu_temp_f64
                                                                    f64.mul
                                                                    local.set $mask_142_y
                                                                    local.set $_rpu_temp_f64
                                                                    local.get $mask_142_x
                                                                    local.get $_rpu_temp_f64
                                                                    f64.mul
                                                                    local.set $mask_142_x
                                                                )
                                                                (else
                                                                    (block
                                                                        (local.get $material_146)
                                                                        (i32.const 8)
                                                                        (i32.add)
                                                                        (f64.load)
                                                                        (local.get $material_146)
                                                                        (i32.const 16)
                                                                        (i32.add)
                                                                        (f64.load)
                                                                        (local.get $material_146)
                                                                        (i32.const 24)
                                                                        (i32.add)
                                                                        (f64.load)
                                                                        local.get $TP_177
                                                                        (call $_rpu_vec3_mul_scalar_f64)
                                                                        local.set $_rpu_temp_f64
                                                                        local.get $mask_142_z
                                                                        local.get $_rpu_temp_f64
                                                                        f64.mul
                                                                        local.set $mask_142_z
                                                                        local.set $_rpu_temp_f64
                                                                        local.get $mask_142_y
                                                                        local.get $_rpu_temp_f64
                                                                        f64.mul
                                                                        local.set $mask_142_y
                                                                        local.set $_rpu_temp_f64
                                                                        local.get $mask_142_x
                                                                        local.get $_rpu_temp_f64
                                                                        f64.mul
                                                                        local.set $mask_142_x
                                                                        local.get $x_147_x
                                                                        local.get $x_147_y
                                                                        local.get $x_147_z
                                                                        local.get $tdir_171_x
                                                                        local.get $tdir_171_y
                                                                        local.get $tdir_171_z
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
                                                                        (local.set $r_140)
                                                                    )
                                                                )
                                                            )
                                                        )
                                                    )
                                                )
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
                (i64.const 1)
                local.get $depth_143
                i64.add
                local.set $depth_143
                (br 0)
            )
        )
        local.get $acc_141_x
        local.get $acc_141_y
        local.get $acc_141_z
        (return)
    )

    ;; function 'shader'
    (func $shader (export "shader") (param $coord_178_x f64) (param $coord_178_y f64)(param $resolution_179_x f64) (param $resolution_179_y f64) (result f64 f64 f64 f64)
        (local $uv_180_x f64)
        (local $uv_180_y f64)
        (local $origin_181_x f64)
        (local $origin_181_y f64)
        (local $origin_181_z f64)
        (local $lookAt_182_x f64)
        (local $lookAt_182_y f64)
        (local $lookAt_182_z f64)
        (local $ray_183 i32)
        (local $color_184_x f64)
        (local $color_184_y f64)
        (local $color_184_z f64)
        (local $_rpu_temp_f64 f64)
        local.get $coord_178_x
        local.get $coord_178_y
        (call $_rpu_rand)
        (call $_rpu_rand)
        (call $_rpu_vec2_add_vec2_f64)
        local.get $resolution_179_x
        local.get $resolution_179_y
        (call $_rpu_vec2_div_vec2_f64)
        local.set $uv_180_y
        local.set $uv_180_x
        (f64.const 13)
        (f64.const 2)
        (f64.const 3)
        local.set $origin_181_z
        local.set $origin_181_y
        local.set $origin_181_x
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $lookAt_182_z
        local.set $lookAt_182_y
        local.set $lookAt_182_x
        local.get $uv_180_x
        local.get $uv_180_y
        local.get $resolution_179_x
        local.get $resolution_179_y
        local.get $origin_181_x
        local.get $origin_181_y
        local.get $origin_181_z
        local.get $lookAt_182_x
        local.get $lookAt_182_y
        local.get $lookAt_182_z
        (call $createRay)
        (local.set $ray_183)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $color_184_z
        local.set $color_184_y
        local.set $color_184_x
        (local.get $ray_183)
        (call $radiance)
        local.set $_rpu_temp_f64
        local.get $color_184_z
        local.get $_rpu_temp_f64
        f64.add
        local.set $color_184_z
        local.set $_rpu_temp_f64
        local.get $color_184_y
        local.get $_rpu_temp_f64
        f64.add
        local.set $color_184_y
        local.set $_rpu_temp_f64
        local.get $color_184_x
        local.get $_rpu_temp_f64
        f64.add
        local.set $color_184_x
        local.get $color_184_x
        local.get $color_184_y
        local.get $color_184_z
        (f64.const 1)
        (f64.const 2.2)
        (f64.div)
        (call $_rpu_vec3_pow_f64)
        (f64.const 1)
        (return)
    )
)
