(module
    (import "env" "_rpu_min" (func $_rpu_min (param f64) (param f64) (result f64)))
    (import "env" "_rpu_max" (func $_rpu_max (param f64) (param f64) (result f64)))
    (import "env" "_rpu_pow" (func $_rpu_pow (param f64) (param f64) (result f64)))
    (import "env" "_rpu_rand" (func $_rpu_rand (result f64)))

    (memory 1)


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

    ;; vec2 div scalar (f64)
    (func $_rpu_vec2_div_scalar_f64
        (param $vec2_x f64)    ;; x component of vec2
        (param $vec2_y f64)    ;; y component of vec2
        (param $scalar f64)    ;; Scalar
        (result f64 f64)       ;; Return two f64 results, the new x and y components

        ;; Calculate the new x component and return it
        (f64.div
            (local.get $vec2_x)  ;; Get the x component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new y component and return it
        (f64.div
            (local.get $vec2_y)  ;; Get the y component
            (local.get $scalar)  ;; Get the scalar
        )
    )

    ;; vec1 neg
    (func $_rpu_vec1_neg_f64  (param $x f64)  (result f64)
        local.get $x
        f64.neg)

    ;; vec1 abs
    (func $_rpu_vec1_abs_f64  (param $x f64)  (result f64)
        local.get $x
        f64.abs)

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

    ;; function 'sdBox'
    (func $sdBox (param $p_0_x f64) (param $p_0_y f64) (param $p_0_z f64)(param $s_1_x f64) (param $s_1_y f64) (param $s_1_z f64) (result f64)
        local.get $p_0_x
        local.get $p_0_y
        local.get $p_0_z
        (call $_rpu_vec3_abs_f64)
        local.get $s_1_x
        local.get $s_1_y
        local.get $s_1_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $p_0_z
        local.set $p_0_y
        local.set $p_0_x
        local.get $p_0_x
        local.get $p_0_y
        local.get $p_0_z
        (f64.const 0)
        (call $_rpu_vec3_max_f64)
        (call $_rpu_vec3_length_f64)
        local.get $p_0_x
        local.get $p_0_y
        local.get $p_0_z
        (call $_rpu_vec1_max_f64)
        (call $_rpu_vec1_max_f64)
        (f64.const 0)
        (call $_rpu_vec1_min_f64)
        (f64.add)
        (return)
    )

    ;; function 'GetDist'
    (func $GetDist (param $p_2_x f64) (param $p_2_y f64) (param $p_2_z f64) (result f64)
        (local $d_3 f64)
        local.get $p_2_x
        local.get $p_2_y
        local.get $p_2_z
        (f64.const 0.5)
        (f64.const 0.5)
        (f64.const 0.5)
        (call $sdBox)
        local.set $d_3
        local.get $d_3
        (return)
    )

    ;; function 'GetRayDir'
    (func $GetRayDir (param $uv_4_x f64) (param $uv_4_y f64)(param $p_5_x f64) (param $p_5_y f64) (param $p_5_z f64)(param $l_6_x f64) (param $l_6_y f64) (param $l_6_z f64)(param $z_7 f64) (result f64 f64 f64)
        (local $f_8_x f64)
        (local $f_8_y f64)
        (local $f_8_z f64)
        (local $r_9_x f64)
        (local $r_9_y f64)
        (local $r_9_z f64)
        (local $u_10_x f64)
        (local $u_10_y f64)
        (local $u_10_z f64)
        (local $c_11_x f64)
        (local $c_11_y f64)
        (local $c_11_z f64)
        (local $i_12_x f64)
        (local $i_12_y f64)
        (local $i_12_z f64)
        local.get $l_6_x
        local.get $l_6_y
        local.get $l_6_z
        local.get $p_5_x
        local.get $p_5_y
        local.get $p_5_z
        (call $_rpu_vec3_sub_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $f_8_z
        local.set $f_8_y
        local.set $f_8_x
        (f64.const 0)
        (f64.const 1)
        (f64.const 0)
        local.get $f_8_x
        local.get $f_8_y
        local.get $f_8_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $r_9_z
        local.set $r_9_y
        local.set $r_9_x
        local.get $f_8_x
        local.get $f_8_y
        local.get $f_8_z
        local.get $r_9_x
        local.get $r_9_y
        local.get $r_9_z
        (call $_rpu_cross_product_f64)
        local.set $u_10_z
        local.set $u_10_y
        local.set $u_10_x
        local.get $f_8_x
        local.get $f_8_y
        local.get $f_8_z
        local.get $z_7
        (call $_rpu_vec3_mul_scalar_f64)
        local.set $c_11_z
        local.set $c_11_y
        local.set $c_11_x
        local.get $c_11_x
        local.get $c_11_y
        local.get $c_11_z
        local.get $uv_4_x
        local.get $r_9_x
        local.get $r_9_y
        local.get $r_9_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $uv_4_y
        local.get $u_10_x
        local.get $u_10_y
        local.get $u_10_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.set $i_12_z
        local.set $i_12_y
        local.set $i_12_x
        local.get $i_12_x
        local.get $i_12_y
        local.get $i_12_z
        (call $_rpu_normalize_vec3_f64)
        (return)
    )

    ;; function 'GetNormal'
    (func $GetNormal (param $p_13_x f64) (param $p_13_y f64) (param $p_13_z f64) (result f64 f64 f64)
        (local $e_14_x f64)
        (local $e_14_y f64)
        (local $n_15_x f64)
        (local $n_15_y f64)
        (local $n_15_z f64)
        (f64.const 0.001)
        (f64.const 0)
        local.set $e_14_y
        local.set $e_14_x
        local.get $p_13_x
        local.get $p_13_y
        local.get $p_13_z
        (call $GetDist)
        local.get $p_13_x
        local.get $p_13_y
        local.get $p_13_z
        local.get $e_14_x
        local.get $e_14_y
        local.get $e_14_y
        (call $_rpu_vec3_sub_vec3_f64)
        (call $GetDist)
        local.get $p_13_x
        local.get $p_13_y
        local.get $p_13_z
        local.get $e_14_y
        local.get $e_14_x
        local.get $e_14_y
        (call $_rpu_vec3_sub_vec3_f64)
        (call $GetDist)
        local.get $p_13_x
        local.get $p_13_y
        local.get $p_13_z
        local.get $e_14_y
        local.get $e_14_y
        local.get $e_14_x
        (call $_rpu_vec3_sub_vec3_f64)
        (call $GetDist)
        (call $_rpu_scalar_sub_vec3_f64)
        local.set $n_15_z
        local.set $n_15_y
        local.set $n_15_x
        local.get $n_15_x
        local.get $n_15_y
        local.get $n_15_z
        (call $_rpu_normalize_vec3_f64)
        (return)
    )

    ;; function 'shader'
    (func $shader (export "shader") (param $coord_16_x f64) (param $coord_16_y f64)(param $resolution_17_x f64) (param $resolution_17_y f64) (result f64 f64 f64 f64)
        (local $uv_18_x f64)
        (local $uv_18_y f64)
        (local $ro_19_x f64)
        (local $ro_19_y f64)
        (local $ro_19_z f64)
        (local $rd_20_x f64)
        (local $rd_20_y f64)
        (local $rd_20_z f64)
        (local $t_21 f64)
        (local $max_t_22 f64)
        (local $col_23_x f64)
        (local $col_23_y f64)
        (local $col_23_z f64)
        (local $col_23_w f64)
        (local $p_24_x f64)
        (local $p_24_y f64)
        (local $p_24_z f64)
        (local $d_25 f64)
        (local $n_26_x f64)
        (local $n_26_y f64)
        (local $n_26_z f64)
        (local $dif_27 f64)
        (f64.const 2)
        local.get $coord_16_x
        local.get $coord_16_y
        (call $_rpu_rand)
        (call $_rpu_rand)
        (call $_rpu_vec2_add_vec2_f64)
        (call $_rpu_scalar_mul_vec2_f64)
        local.get $resolution_17_x
        local.get $resolution_17_y
        (call $_rpu_vec2_sub_vec2_f64)
        local.get $resolution_17_y
        (call $_rpu_vec2_div_scalar_f64)
        local.set $uv_18_y
        local.set $uv_18_x
        (f64.const 0.7)
        (f64.const 0.8)
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        local.set $ro_19_z
        local.set $ro_19_y
        local.set $ro_19_x
        local.get $uv_18_x
        local.get $uv_18_y
        local.get $ro_19_x
        local.get $ro_19_y
        local.get $ro_19_z
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 1)
        (call $GetRayDir)
        local.set $rd_20_z
        local.set $rd_20_y
        local.set $rd_20_x
        (f64.const 0)
        local.set $t_21
        (f64.const 2)
        local.set $max_t_22
        local.get $uv_18_x
        local.get $uv_18_y
        (f64.const 0)
        (f64.const 1)
        local.set $col_23_w
        local.set $col_23_z
        local.set $col_23_y
        local.set $col_23_x

        (block
            (loop
                local.get $t_21
                local.get $max_t_22
                (f64.lt)
                (i32.eqz)
                (br_if 1)
                (block
                    local.get $ro_19_x
                    local.get $ro_19_y
                    local.get $ro_19_z
                    local.get $rd_20_x
                    local.get $rd_20_y
                    local.get $rd_20_z
                    local.get $t_21
                    (call $_rpu_vec3_mul_scalar_f64)
                    (call $_rpu_vec3_add_vec3_f64)
                    local.set $p_24_z
                    local.set $p_24_y
                    local.set $p_24_x
                    local.get $p_24_x
                    local.get $p_24_y
                    local.get $p_24_z
                    (call $GetDist)
                    local.set $d_25

                    local.get $d_25
                    (call $_rpu_vec1_abs_f64)
                    (f64.const 0.001)
                    (f64.lt)
                    (if
                        (then
                            (block
                                local.get $p_24_x
                                local.get $p_24_y
                                local.get $p_24_z
                                (call $GetNormal)
                                local.set $n_26_z
                                local.set $n_26_y
                                local.set $n_26_x
                                local.get $n_26_x
                                local.get $n_26_y
                                local.get $n_26_z
                                (f64.const 1)
                                (f64.const 2)
                                (f64.const 3)
                                (call $_rpu_normalize_vec3_f64)
                                (call $_rpu_dot_product_vec3_f64)
                                (f64.const 0.5)
                                (f64.mul)
                                (f64.const 0.5)
                                (f64.add)
                                local.set $dif_27
                                local.get $dif_27
                                local.get $dif_27
                                local.get $dif_27
                                (f64.const 0.4545)
                                (call $_rpu_vec3_pow_f64)
                                local.set $col_23_z
                                local.set $col_23_y
                                local.set $col_23_x
                                (br 4)
                            )
                        )
                    )
                    local.get $d_25
                    local.get $t_21
                    f64.add
                    local.set $t_21
                )
                (br 0)
            )
        )
        local.get $col_23_x
        local.get $col_23_y
        local.get $col_23_z
        local.get $col_23_w
        (return)
    )
)
