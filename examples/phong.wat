(module
    (import "env" "_rpu_sin" (func $_rpu_sin (param f64) (result f64)))
    (import "env" "_rpu_cos" (func $_rpu_cos (param f64) (result f64)))
    (import "env" "_rpu_pow" (func $_rpu_pow (param f64) (param f64) (result f64)))
    (import "env" "_rpu_rand" (func $_rpu_rand (result f64)))
    (import "env" "_rpu_clamp" (func $_rpu_clamp (param f64) (param f64) (param f64) (result f64)))

    (memory 1)

    (global $lightDir_3_x (mut f64) (f64.const 0.0))
    (global $lightDir_3_y (mut f64) (f64.const 4.0))
    (global $lightDir_3_z (mut f64) (f64.const 5.0))
    (global $specularColor_2_x (mut f64) (f64.const 1.0))
    (global $specularColor_2_y (mut f64) (f64.const 1.0))
    (global $specularColor_2_z (mut f64) (f64.const 1.0))
    (global $diffuseColor_1_x (mut f64) (f64.const 0.2))
    (global $diffuseColor_1_y (mut f64) (f64.const 0.6))
    (global $diffuseColor_1_z (mut f64) (f64.const 0.8))
    (global $spherePos_4_x (mut f64) (f64.const 0.0))
    (global $spherePos_4_y (mut f64) (f64.const 0.5))
    (global $spherePos_4_z (mut f64) (f64.const 0.0))
    (global $ambientColor_0_x (mut f64) (f64.const 0.05))
    (global $ambientColor_0_y (mut f64) (f64.const 0.15))
    (global $ambientColor_0_z (mut f64) (f64.const 0.2))

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

    ;; vec1 neg
    (func $_rpu_vec1_neg_f64  (param $x f64)  (result f64)
        local.get $x
        f64.neg)

    ;; vec1 sqrt
    (func $_rpu_vec1_sqrt_f64  (param $x f64)  (result f64)
        local.get $x
        f64.sqrt)

    ;; vec2 neg
    (func $_rpu_vec2_neg_f64  (param $x f64)  (param $y f64)  (result f64 f64)
        local.get $x
        f64.neg
        local.get $y
        f64.neg)

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

    ;; vec1 cos
    (func $_rpu_vec1_cos_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_cos))

    ;; vec1 sin
    (func $_rpu_vec1_sin_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_sin))

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

    ;; mat3 mul vec3 (f64)
    (func $_rpu_mat3_mul_vec3_f64
        (param $a f64)  ;; Matrix component a (row 1, col 1)
        (param $b f64)  ;; Matrix component b (row 1, col 2)
        (param $c f64)  ;; Matrix component c (row 1, col 3)
        (param $d f64)  ;; Matrix component d (row 2, col 1)
        (param $e f64)  ;; Matrix component e (row 2, col 2)
        (param $f f64)  ;; Matrix component f (row 2, col 3)
        (param $g f64)  ;; Matrix component g (row 3, col 1)
        (param $h f64)  ;; Matrix component h (row 3, col 2)
        (param $i f64)  ;; Matrix component i (row 3, col 3)
        (param $x f64)  ;; Vector component x
        (param $y f64)  ;; Vector component y
        (param $z f64)  ;; Vector component z
        (result f64 f64 f64) ;; Resulting vector components

        ;; Compute the first component of the resulting vector: a*x + b*y + c*z
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

        ;; Compute the second component of the resulting vector: d*x + e*y + f*z
        local.get $d
        local.get $x
        f64.mul
        local.get $e
        local.get $y
        f64.mul
        f64.add
        local.get $f
        local.get $z
        f64.mul
        f64.add

        ;; Compute the third component of the resulting vector: g*x + h*y + i*z
        local.get $g
        local.get $x
        f64.mul
        local.get $h
        local.get $y
        f64.mul
        f64.add
        local.get $i
        local.get $z
        f64.mul
        f64.add
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

    ;; vec1 clamp
    (func $_rpu_vec1_clamp_f64_f64  (param $x f64)  (param $scalar f64) (param $scalar2 f64) (result f64)
        local.get $x
        local.get $scalar
        local.get $scalar2
        (call $_rpu_clamp))

    ;; vec3 neg
    (func $_rpu_vec3_neg_f64  (param $x f64)  (param $y f64)  (param $z f64)  (result f64 f64 f64)
        local.get $x
        f64.neg
        local.get $y
        f64.neg
        local.get $z
        f64.neg)

    ;; vec1 pow
    (func $_rpu_vec1_pow_f64  (param $x f64)  (param $scalar f64)  (result f64)
        local.get $x
        local.get $scalar
        (call $_rpu_pow))

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

    ;; function 'raytraceSphere'
    (func $raytraceSphere (param $ro_5_x f64) (param $ro_5_y f64) (param $ro_5_z f64)(param $rd_6_x f64) (param $rd_6_y f64) (param $rd_6_z f64)(param $tmin_7 f64)(param $tmax_8 f64)(param $r_9 f64) (result f64)
        (local $ce_10_x f64)
        (local $ce_10_y f64)
        (local $ce_10_z f64)
        (local $b_11 f64)
        (local $c_12 f64)
        (local $t_13 f64)
        local.get $ro_5_x
        local.get $ro_5_y
        local.get $ro_5_z
        global.get $spherePos_4_x
        global.get $spherePos_4_y
        global.get $spherePos_4_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $ce_10_z
        local.set $ce_10_y
        local.set $ce_10_x
        local.get $rd_6_x
        local.get $rd_6_y
        local.get $rd_6_z
        local.get $ce_10_x
        local.get $ce_10_y
        local.get $ce_10_z
        (call $_rpu_dot_product_vec3_f64)
        local.set $b_11
        local.get $ce_10_x
        local.get $ce_10_y
        local.get $ce_10_z
        local.get $ce_10_x
        local.get $ce_10_y
        local.get $ce_10_z
        (call $_rpu_dot_product_vec3_f64)
        local.get $r_9
        local.get $r_9
        (f64.mul)
        (f64.sub)
        local.set $c_12
        local.get $b_11
        local.get $b_11
        (f64.mul)
        local.get $c_12
        (f64.sub)
        local.set $t_13

        local.get $t_13
        local.get $tmin_7
        (f64.gt)
        (if
            (then
                (block
                    local.get $b_11
                    (call $_rpu_vec1_neg_f64)
                    local.get $t_13
                    (call $_rpu_vec1_sqrt_f64)
                    (f64.sub)
                    local.set $t_13

                    local.get $t_13
                    local.get $tmax_8
                    (f64.lt)
                    (if
                        (then
                            local.get $t_13
                            (return)
                        )
                    )
                )
            )
        )
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        (return)
    )

    ;; function 'shader'
    (func $shader (export "shader") (param $coord_14_x f64) (param $coord_14_y f64)(param $resolution_15_x f64) (param $resolution_15_y f64) (result f64 f64 f64 f64)
        (local $p_16_x f64)
        (local $p_16_y f64)
        (local $eye_17_x f64)
        (local $eye_17_y f64)
        (local $eye_17_z f64)
        (local $rot_18_x f64)
        (local $rot_18_y f64)
        (local $ro_19_x f64)
        (local $ro_19_y f64)
        (local $ro_19_z f64)
        (local $ta_20_x f64)
        (local $ta_20_y f64)
        (local $ta_20_z f64)
        (local $cw_21_x f64)
        (local $cw_21_y f64)
        (local $cw_21_z f64)
        (local $cu_22_x f64)
        (local $cu_22_y f64)
        (local $cu_22_z f64)
        (local $cv_23_x f64)
        (local $cv_23_y f64)
        (local $cv_23_z f64)
        (local $cam_24_1 f64)
        (local $cam_24_2 f64)
        (local $cam_24_3 f64)
        (local $cam_24_4 f64)
        (local $cam_24_5 f64)
        (local $cam_24_6 f64)
        (local $cam_24_7 f64)
        (local $cam_24_8 f64)
        (local $cam_24_9 f64)
        (local $rd_25_x f64)
        (local $rd_25_y f64)
        (local $rd_25_z f64)
        (local $color_26_x f64)
        (local $color_26_y f64)
        (local $color_26_z f64)
        (local $tmin_27 f64)
        (local $tmax_28 f64)
        (local $t_29 f64)
        (local $pos_30_x f64)
        (local $pos_30_y f64)
        (local $pos_30_z f64)
        (local $norm_31_x f64)
        (local $norm_31_y f64)
        (local $norm_31_z f64)
        (local $occ_32 f64)
        (local $amb_33 f64)
        (local $dif_34 f64)
        (local $h_35_x f64)
        (local $h_35_y f64)
        (local $h_35_z f64)
        (local $spe_36 f64)
        (local $_rpu_temp_f64 f64)
        local.get $resolution_15_x
        local.get $resolution_15_y
        (call $_rpu_vec2_neg_f64)
        (f64.const 2)
        local.get $coord_14_x
        local.get $coord_14_y
        (call $_rpu_rand)
        (call $_rpu_rand)
        (call $_rpu_vec2_add_vec2_f64)
        (call $_rpu_scalar_mul_vec2_f64)
        (call $_rpu_vec2_add_vec2_f64)
        local.get $resolution_15_y
        (call $_rpu_vec2_div_scalar_f64)
        local.set $p_16_y
        local.set $p_16_x
        (f64.const 0)
        (f64.const 1)
        (f64.const 2)
        local.set $eye_17_z
        local.set $eye_17_y
        local.set $eye_17_x
        (f64.const 6.2831)
        (f64.const 0.1)
        (f64.const 150)
        (f64.const 0.25)
        (f64.mul)
        (f64.add)
        (f64.const 0)
        (f64.const 1)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.get $resolution_15_x
        local.get $resolution_15_y
        (f64.const 0.25)
        (call $_rpu_vec2_mul_scalar_f64)
        (call $_rpu_vec2_sub_vec2_f64)
        (call $_rpu_vec2_mul_vec2_f64)
        local.get $resolution_15_x
        (call $_rpu_vec2_div_scalar_f64)
        (call $_rpu_vec2_add_vec2_f64)
        (call $_rpu_scalar_mul_vec2_f64)
        local.set $rot_18_y
        local.set $rot_18_x
        local.get $rot_18_y
        (call $_rpu_vec1_cos_f64)
        local.get $eye_17_y
        local.get $eye_17_z
        (call $_rpu_scalar_mul_vec2_f64)
        local.get $rot_18_y
        (call $_rpu_vec1_sin_f64)
        local.get $eye_17_z
        local.get $eye_17_y
        (call $_rpu_scalar_mul_vec2_f64)
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        (f64.const 1)
        (call $_rpu_vec2_mul_vec2_f64)
        (call $_rpu_vec2_add_vec2_f64)
        local.set $eye_17_z
        local.set $eye_17_y
        local.get $eye_17_x
        local.get $eye_17_y
        local.get $eye_17_z
        local.set $ro_19_z
        local.set $ro_19_y
        local.set $ro_19_x
        (f64.const 0)
        (f64.const 0.5)
        (f64.const 0)
        local.set $ta_20_z
        local.set $ta_20_y
        local.set $ta_20_x
        local.get $ta_20_x
        local.get $ta_20_y
        local.get $ta_20_z
        local.get $eye_17_x
        local.get $eye_17_y
        local.get $eye_17_z
        (call $_rpu_vec3_sub_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $cw_21_z
        local.set $cw_21_y
        local.set $cw_21_x
        (f64.const 0)
        (f64.const 1)
        (f64.const 0)
        local.get $cw_21_x
        local.get $cw_21_y
        local.get $cw_21_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $cu_22_z
        local.set $cu_22_y
        local.set $cu_22_x
        local.get $cw_21_x
        local.get $cw_21_y
        local.get $cw_21_z
        local.get $cu_22_x
        local.get $cu_22_y
        local.get $cu_22_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $cv_23_z
        local.set $cv_23_y
        local.set $cv_23_x
        local.get $cu_22_x
        local.get $cu_22_y
        local.get $cu_22_z
        local.get $cv_23_x
        local.get $cv_23_y
        local.get $cv_23_z
        local.get $cw_21_x
        local.get $cw_21_y
        local.get $cw_21_z
        (local.set $cam_24_9)
        (local.set $cam_24_8)
        (local.set $cam_24_7)
        (local.set $cam_24_6)
        (local.set $cam_24_5)
        (local.set $cam_24_4)
        (local.set $cam_24_3)
        (local.set $cam_24_2)
        (local.set $cam_24_1)
        (local.get $cam_24_1)
        (local.get $cam_24_2)
        (local.get $cam_24_3)
        (local.get $cam_24_4)
        (local.get $cam_24_5)
        (local.get $cam_24_6)
        (local.get $cam_24_7)
        (local.get $cam_24_8)
        (local.get $cam_24_9)
        local.get $p_16_x
        local.get $p_16_y
        (f64.const 1.5)
        (call $_rpu_normalize_vec3_f64)
        (call $_rpu_mat3_mul_vec3_f64)
        local.set $rd_25_z
        local.set $rd_25_y
        local.set $rd_25_x
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $color_26_z
        local.set $color_26_y
        local.set $color_26_x
        (f64.const 0.1)
        local.set $tmin_27
        (f64.const 50)
        local.set $tmax_28
        local.get $ro_19_x
        local.get $ro_19_y
        local.get $ro_19_z
        local.get $rd_25_x
        local.get $rd_25_y
        local.get $rd_25_z
        local.get $tmin_27
        local.get $tmax_28
        (f64.const 1)
        (call $raytraceSphere)
        local.set $t_29

        local.get $t_29
        local.get $tmin_27
        (f64.gt)
        local.get $t_29
        local.get $tmax_28
        (f64.lt)
        (i32.and)
        (if
            (then
                (block
                    local.get $ro_19_x
                    local.get $ro_19_y
                    local.get $ro_19_z
                    local.get $rd_25_x
                    local.get $rd_25_y
                    local.get $rd_25_z
                    local.get $t_29
                    (call $_rpu_vec3_mul_scalar_f64)
                    (call $_rpu_vec3_add_vec3_f64)
                    local.set $pos_30_z
                    local.set $pos_30_y
                    local.set $pos_30_x
                    local.get $pos_30_x
                    local.get $pos_30_y
                    local.get $pos_30_z
                    global.get $spherePos_4_x
                    global.get $spherePos_4_y
                    global.get $spherePos_4_z
                    (call $_rpu_vec3_sub_vec3_f64)
                    (call $_rpu_normalize_vec3_f64)
                    local.set $norm_31_z
                    local.set $norm_31_y
                    local.set $norm_31_x
                    (f64.const 0.5)
                    (f64.const 0.5)
                    local.get $norm_31_y
                    (f64.mul)
                    (f64.add)
                    local.set $occ_32
                    (f64.const 0.5)
                    (f64.const 0.5)
                    local.get $norm_31_y
                    (f64.mul)
                    (f64.add)
                    (f64.const 0)
                    (f64.const 1)
                    (call $_rpu_vec1_clamp_f64_f64)
                    local.set $amb_33
                    global.get $lightDir_3_x
                    global.get $lightDir_3_y
                    global.get $lightDir_3_z
                    local.get $norm_31_x
                    local.get $norm_31_y
                    local.get $norm_31_z
                    (call $_rpu_dot_product_vec3_f64)
                    (f64.const 0)
                    (f64.const 1)
                    (call $_rpu_vec1_clamp_f64_f64)
                    local.set $dif_34
                    local.get $rd_25_x
                    local.get $rd_25_y
                    local.get $rd_25_z
                    (call $_rpu_vec3_neg_f64)
                    global.get $lightDir_3_x
                    global.get $lightDir_3_y
                    global.get $lightDir_3_z
                    (call $_rpu_vec3_add_vec3_f64)
                    (call $_rpu_normalize_vec3_f64)
                    local.set $h_35_z
                    local.set $h_35_y
                    local.set $h_35_x
                    local.get $h_35_x
                    local.get $h_35_y
                    local.get $h_35_z
                    local.get $norm_31_x
                    local.get $norm_31_y
                    local.get $norm_31_z
                    (call $_rpu_dot_product_vec3_f64)
                    (f64.const 0)
                    (f64.const 1)
                    (call $_rpu_vec1_clamp_f64_f64)
                    (f64.const 64)
                    (call $_rpu_vec1_pow_f64)
                    local.set $spe_36
                    local.get $amb_33
                    global.get $ambientColor_0_x
                    global.get $ambientColor_0_y
                    global.get $ambientColor_0_z
                    (call $_rpu_scalar_mul_vec3_f64)
                    local.get $occ_32
                    (call $_rpu_vec3_mul_scalar_f64)
                    local.set $color_26_z
                    local.set $color_26_y
                    local.set $color_26_x
                    local.get $dif_34
                    global.get $diffuseColor_1_x
                    global.get $diffuseColor_1_y
                    global.get $diffuseColor_1_z
                    (call $_rpu_scalar_mul_vec3_f64)
                    local.get $occ_32
                    (call $_rpu_vec3_mul_scalar_f64)
                    local.set $_rpu_temp_f64
                    local.get $color_26_z
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $color_26_z
                    local.set $_rpu_temp_f64
                    local.get $color_26_y
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $color_26_y
                    local.set $_rpu_temp_f64
                    local.get $color_26_x
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $color_26_x
                    local.get $dif_34
                    local.get $spe_36
                    (f64.mul)
                    global.get $specularColor_2_x
                    global.get $specularColor_2_y
                    global.get $specularColor_2_z
                    (call $_rpu_scalar_mul_vec3_f64)
                    local.get $occ_32
                    (call $_rpu_vec3_mul_scalar_f64)
                    local.set $_rpu_temp_f64
                    local.get $color_26_z
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $color_26_z
                    local.set $_rpu_temp_f64
                    local.get $color_26_y
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $color_26_y
                    local.set $_rpu_temp_f64
                    local.get $color_26_x
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $color_26_x
                )
            )
        )
        local.get $color_26_x
        local.get $color_26_y
        local.get $color_26_z
        (f64.const 1)
        (f64.const 2.2)
        (f64.div)
        (call $_rpu_vec3_pow_f64)
        (f64.const 1)
        (return)
    )
)
