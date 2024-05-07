(module
    (import "env" "_rpu_sin" (func $_rpu_sin (param f64) (result f64)))
    (import "env" "_rpu_cos" (func $_rpu_cos (param f64) (result f64)))
    (import "env" "_rpu_pow" (func $_rpu_pow (param f64) (param f64) (result f64)))
    (import "env" "_rpu_rand" (func $_rpu_rand (result f64)))
    (import "env" "_rpu_clamp" (func $_rpu_clamp (param f64) (param f64) (param f64) (result f64)))

    (memory 1)
    (global $diffuseColor_x (mut f64) (f64.const 0.2))
    (global $diffuseColor_y (mut f64) (f64.const 0.6))
    (global $diffuseColor_z (mut f64) (f64.const 0.8))
    (global $spherePos_x (mut f64) (f64.const 0.0))
    (global $spherePos_y (mut f64) (f64.const 0.5))
    (global $spherePos_z (mut f64) (f64.const 0.0))
    (global $lightDir_x (mut f64) (f64.const 0.0))
    (global $lightDir_y (mut f64) (f64.const 4.0))
    (global $lightDir_z (mut f64) (f64.const 5.0))
    (global $ambientColor_x (mut f64) (f64.const 0.05))
    (global $ambientColor_y (mut f64) (f64.const 0.15))
    (global $ambientColor_z (mut f64) (f64.const 0.2))
    (global $specularColor_x (mut f64) (f64.const 1.0))
    (global $specularColor_y (mut f64) (f64.const 1.0))
    (global $specularColor_z (mut f64) (f64.const 1.0))

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
    (func $raytraceSphere (param $ro_x f64) (param $ro_y f64) (param $ro_z f64)(param $rd_x f64) (param $rd_y f64) (param $rd_z f64)(param $tmin f64)(param $tmax f64)(param $r f64) (result f64)
        (local $ce_x f64)
        (local $ce_y f64)
        (local $ce_z f64)
        (local $b f64)
        (local $c f64)
        (local $t f64)
        local.get $ro_x
        local.get $ro_y
        local.get $ro_z
        global.get $spherePos_x
        global.get $spherePos_y
        global.get $spherePos_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $ce_z
        local.set $ce_y
        local.set $ce_x
        local.get $rd_x
        local.get $rd_y
        local.get $rd_z
        local.get $ce_x
        local.get $ce_y
        local.get $ce_z
        (call $_rpu_dot_product_vec3_f64)
        local.set $b
        local.get $ce_x
        local.get $ce_y
        local.get $ce_z
        local.get $ce_x
        local.get $ce_y
        local.get $ce_z
        (call $_rpu_dot_product_vec3_f64)
        local.get $r
        local.get $r
        (f64.mul)
        (f64.sub)
        local.set $c
        local.get $b
        local.get $b
        (f64.mul)
        local.get $c
        (f64.sub)
        local.set $t

        local.get $t
        local.get $tmin
        (f64.gt)
        (if
            (then
                (block
                    local.get $b
                    (call $_rpu_vec1_neg_f64)
                    local.get $t
                    (call $_rpu_vec1_sqrt_f64)
                    (f64.sub)
                    local.set $t

                    local.get $t
                    local.get $tmax
                    (f64.lt)
                    (if
                        (then
                            local.get $t
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
    (func $shader (export "shader") (param $coord_x f64) (param $coord_y f64)(param $resolution_x f64) (param $resolution_y f64) (result f64 f64 f64 f64)
        (local $p_x f64)
        (local $p_y f64)
        (local $eye_x f64)
        (local $eye_y f64)
        (local $eye_z f64)
        (local $rot_x f64)
        (local $rot_y f64)
        (local $ro_x f64)
        (local $ro_y f64)
        (local $ro_z f64)
        (local $ta_x f64)
        (local $ta_y f64)
        (local $ta_z f64)
        (local $cw_x f64)
        (local $cw_y f64)
        (local $cw_z f64)
        (local $cu_x f64)
        (local $cu_y f64)
        (local $cu_z f64)
        (local $cv_x f64)
        (local $cv_y f64)
        (local $cv_z f64)
        (local $cam_1 f64)
        (local $cam_2 f64)
        (local $cam_3 f64)
        (local $cam_4 f64)
        (local $cam_5 f64)
        (local $cam_6 f64)
        (local $cam_7 f64)
        (local $cam_8 f64)
        (local $cam_9 f64)
        (local $rd_x f64)
        (local $rd_y f64)
        (local $rd_z f64)
        (local $color_x f64)
        (local $color_y f64)
        (local $color_z f64)
        (local $tmin f64)
        (local $tmax f64)
        (local $t f64)
        (local $pos_x f64)
        (local $pos_y f64)
        (local $pos_z f64)
        (local $norm_x f64)
        (local $norm_y f64)
        (local $norm_z f64)
        (local $occ f64)
        (local $amb f64)
        (local $dif f64)
        (local $h_x f64)
        (local $h_y f64)
        (local $h_z f64)
        (local $spe f64)
        (local $_rpu_temp_f64 f64)
        local.get $resolution_x
        local.get $resolution_y
        (call $_rpu_vec2_neg_f64)
        (f64.const 2)
        local.get $coord_x
        local.get $coord_y
        (call $_rpu_rand)
        (call $_rpu_rand)
        (call $_rpu_vec2_add_vec2_f64)
        (call $_rpu_scalar_mul_vec2_f64)
        (call $_rpu_vec2_add_vec2_f64)
        local.get $resolution_y
        (call $_rpu_vec2_div_scalar_f64)
        local.set $p_y
        local.set $p_x
        (f64.const 0)
        (f64.const 1)
        (f64.const 2)
        local.set $eye_z
        local.set $eye_y
        local.set $eye_x
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
        local.get $resolution_x
        local.get $resolution_y
        (f64.const 0.25)
        (call $_rpu_vec2_mul_scalar_f64)
        (call $_rpu_vec2_sub_vec2_f64)
        (call $_rpu_vec2_mul_vec2_f64)
        local.get $resolution_x
        (call $_rpu_vec2_div_scalar_f64)
        (call $_rpu_vec2_add_vec2_f64)
        (call $_rpu_scalar_mul_vec2_f64)
        local.set $rot_y
        local.set $rot_x
        local.get $rot_y
        (call $_rpu_vec1_cos_f64)
        local.get $eye_y
        local.get $eye_z
        (call $_rpu_scalar_mul_vec2_f64)
        local.get $rot_y
        (call $_rpu_vec1_sin_f64)
        local.get $eye_z
        local.get $eye_y
        (call $_rpu_scalar_mul_vec2_f64)
        (f64.const 1)
        (call $_rpu_vec1_neg_f64)
        (f64.const 1)
        (call $_rpu_vec2_mul_vec2_f64)
        (call $_rpu_vec2_add_vec2_f64)
        local.set $eye_z
        local.set $eye_y
        local.get $eye_x
        local.get $eye_y
        local.get $eye_z
        local.set $ro_z
        local.set $ro_y
        local.set $ro_x
        (f64.const 0)
        (f64.const 0.5)
        (f64.const 0)
        local.set $ta_z
        local.set $ta_y
        local.set $ta_x
        local.get $ta_x
        local.get $ta_y
        local.get $ta_z
        local.get $eye_x
        local.get $eye_y
        local.get $eye_z
        (call $_rpu_vec3_sub_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $cw_z
        local.set $cw_y
        local.set $cw_x
        (f64.const 0)
        (f64.const 1)
        (f64.const 0)
        local.get $cw_x
        local.get $cw_y
        local.get $cw_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $cu_z
        local.set $cu_y
        local.set $cu_x
        local.get $cw_x
        local.get $cw_y
        local.get $cw_z
        local.get $cu_x
        local.get $cu_y
        local.get $cu_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $cv_z
        local.set $cv_y
        local.set $cv_x
        local.get $cu_x
        local.get $cu_y
        local.get $cu_z
        local.get $cv_x
        local.get $cv_y
        local.get $cv_z
        local.get $cw_x
        local.get $cw_y
        local.get $cw_z
        (local.set $cam_9)
        (local.set $cam_8)
        (local.set $cam_7)
        (local.set $cam_6)
        (local.set $cam_5)
        (local.set $cam_4)
        (local.set $cam_3)
        (local.set $cam_2)
        (local.set $cam_1)
        (local.get $cam_1)
        (local.get $cam_2)
        (local.get $cam_3)
        (local.get $cam_4)
        (local.get $cam_5)
        (local.get $cam_6)
        (local.get $cam_7)
        (local.get $cam_8)
        (local.get $cam_9)
        local.get $p_x
        local.get $p_y
        (f64.const 1.5)
        (call $_rpu_normalize_vec3_f64)
        (call $_rpu_mat3_mul_vec3_f64)
        local.set $rd_z
        local.set $rd_y
        local.set $rd_x
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        local.set $color_z
        local.set $color_y
        local.set $color_x
        (f64.const 0.1)
        local.set $tmin
        (f64.const 50)
        local.set $tmax
        local.get $ro_x
        local.get $ro_y
        local.get $ro_z
        local.get $rd_x
        local.get $rd_y
        local.get $rd_z
        local.get $tmin
        local.get $tmax
        (f64.const 1)
        (call $raytraceSphere)
        local.set $t

        local.get $t
        local.get $tmin
        (f64.gt)
        local.get $t
        local.get $tmax
        (f64.lt)
        (i32.and)
        (if
            (then
                (block
                    local.get $ro_x
                    local.get $ro_y
                    local.get $ro_z
                    local.get $rd_x
                    local.get $rd_y
                    local.get $rd_z
                    local.get $t
                    (call $_rpu_vec3_mul_scalar_f64)
                    (call $_rpu_vec3_add_vec3_f64)
                    local.set $pos_z
                    local.set $pos_y
                    local.set $pos_x
                    local.get $pos_x
                    local.get $pos_y
                    local.get $pos_z
                    global.get $spherePos_x
                    global.get $spherePos_y
                    global.get $spherePos_z
                    (call $_rpu_vec3_sub_vec3_f64)
                    (call $_rpu_normalize_vec3_f64)
                    local.set $norm_z
                    local.set $norm_y
                    local.set $norm_x
                    (f64.const 0.5)
                    (f64.const 0.5)
                    local.get $norm_y
                    (f64.mul)
                    (f64.add)
                    local.set $occ
                    (f64.const 0.5)
                    (f64.const 0.5)
                    local.get $norm_y
                    (f64.mul)
                    (f64.add)
                    (f64.const 0)
                    (f64.const 1)
                    (call $_rpu_vec1_clamp_f64_f64)
                    local.set $amb
                    global.get $lightDir_x
                    global.get $lightDir_y
                    global.get $lightDir_z
                    local.get $norm_x
                    local.get $norm_y
                    local.get $norm_z
                    (call $_rpu_dot_product_vec3_f64)
                    (f64.const 0)
                    (f64.const 1)
                    (call $_rpu_vec1_clamp_f64_f64)
                    local.set $dif
                    local.get $rd_x
                    local.get $rd_y
                    local.get $rd_z
                    (call $_rpu_vec3_neg_f64)
                    global.get $lightDir_x
                    global.get $lightDir_y
                    global.get $lightDir_z
                    (call $_rpu_vec3_add_vec3_f64)
                    (call $_rpu_normalize_vec3_f64)
                    local.set $h_z
                    local.set $h_y
                    local.set $h_x
                    local.get $h_x
                    local.get $h_y
                    local.get $h_z
                    local.get $norm_x
                    local.get $norm_y
                    local.get $norm_z
                    (call $_rpu_dot_product_vec3_f64)
                    (f64.const 0)
                    (f64.const 1)
                    (call $_rpu_vec1_clamp_f64_f64)
                    (f64.const 64)
                    (call $_rpu_vec1_pow_f64)
                    local.set $spe
                    local.get $amb
                    global.get $ambientColor_x
                    global.get $ambientColor_y
                    global.get $ambientColor_z
                    (call $_rpu_scalar_mul_vec3_f64)
                    local.get $occ
                    (call $_rpu_vec3_mul_scalar_f64)
                    local.set $color_z
                    local.set $color_y
                    local.set $color_x
                    local.get $dif
                    global.get $diffuseColor_x
                    global.get $diffuseColor_y
                    global.get $diffuseColor_z
                    (call $_rpu_scalar_mul_vec3_f64)
                    local.get $occ
                    (call $_rpu_vec3_mul_scalar_f64)
                    local.set $_rpu_temp_f64
                    local.get $color_z
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $color_z
                    local.set $_rpu_temp_f64
                    local.get $color_y
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $color_y
                    local.set $_rpu_temp_f64
                    local.get $color_x
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $color_x
                    local.get $dif
                    local.get $spe
                    (f64.mul)
                    global.get $specularColor_x
                    global.get $specularColor_y
                    global.get $specularColor_z
                    (call $_rpu_scalar_mul_vec3_f64)
                    local.get $occ
                    (call $_rpu_vec3_mul_scalar_f64)
                    local.set $_rpu_temp_f64
                    local.get $color_z
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $color_z
                    local.set $_rpu_temp_f64
                    local.get $color_y
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $color_y
                    local.set $_rpu_temp_f64
                    local.get $color_x
                    local.get $_rpu_temp_f64
                    f64.add
                    local.set $color_x
                )
            )
        )
        local.get $color_x
        local.get $color_y
        local.get $color_z
        (f64.const 1)
        (f64.const 2.2)
        (f64.div)
        (call $_rpu_vec3_pow_f64)
        (f64.const 1)
        (return)
    )
)
