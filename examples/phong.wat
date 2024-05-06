(module
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

    ;; function 'GetRayDir'
    (func $GetRayDir (param $uv_x f64) (param $uv_y f64)(param $p_x f64) (param $p_y f64) (param $p_z f64)(param $l_x f64) (param $l_y f64) (param $l_z f64)(param $z f64) (result f64 f64 f64)
        (local $f_x f64)
        (local $f_y f64)
        (local $f_z f64)
        (local $r_x f64)
        (local $r_y f64)
        (local $r_z f64)
        (local $u_x f64)
        (local $u_y f64)
        (local $u_z f64)
        (local $c_x f64)
        (local $c_y f64)
        (local $c_z f64)
        (local $i_x f64)
        (local $i_y f64)
        (local $i_z f64)
        local.get $l_x
        local.get $l_y
        local.get $l_z
        local.get $p_x
        local.get $p_y
        local.get $p_z
        (call $_rpu_vec3_sub_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $f_z
        local.set $f_y
        local.set $f_x
        (f64.const 0)
        (f64.const 1)
        (f64.const 0)
        local.get $f_x
        local.get $f_y
        local.get $f_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $r_z
        local.set $r_y
        local.set $r_x
        local.get $f_x
        local.get $f_y
        local.get $f_z
        local.get $r_x
        local.get $r_y
        local.get $r_z
        (call $_rpu_cross_product_f64)
        local.set $u_z
        local.set $u_y
        local.set $u_x
        local.get $f_x
        local.get $f_y
        local.get $f_z
        local.get $z
        (call $_rpu_vec3_mul_scalar_f64)
        local.set $c_z
        local.set $c_y
        local.set $c_x
        local.get $c_x
        local.get $c_y
        local.get $c_z
        local.get $uv_x
        local.get $r_x
        local.get $r_y
        local.get $r_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $uv_y
        local.get $u_x
        local.get $u_y
        local.get $u_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.set $i_z
        local.set $i_y
        local.set $i_x
        local.get $i_x
        local.get $i_y
        local.get $i_z
        (call $_rpu_normalize_vec3_f64)
        (return)
    )

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
        (local $uv_x f64)
        (local $uv_y f64)
        (local $ro_x f64)
        (local $ro_y f64)
        (local $ro_z f64)
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
        (local $c_x f64)
        (local $c_y f64)
        (local $c_z f64)
        (f64.const 2)
        local.get $coord_x
        local.get $coord_y
        (call $_rpu_rand)
        (call $_rpu_rand)
        (call $_rpu_vec2_add_vec2_f64)
        (call $_rpu_scalar_mul_vec2_f64)
        local.get $resolution_x
        local.get $resolution_y
        (call $_rpu_vec2_sub_vec2_f64)
        local.get $resolution_y
        (call $_rpu_vec2_div_scalar_f64)
        local.set $uv_y
        local.set $uv_x
        (f64.const 0.5)
        (f64.const 0.8)
        (f64.const 3)
        local.set $ro_z
        local.set $ro_y
        local.set $ro_x
        local.get $uv_x
        local.get $uv_y
        local.get $ro_x
        local.get $ro_y
        local.get $ro_z
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 1)
        (call $GetRayDir)
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
                    local.get $color_x
                    local.get $color_y
                    local.get $color_z
                    local.get $dif
                    global.get $diffuseColor_x
                    global.get $diffuseColor_y
                    global.get $diffuseColor_z
                    (call $_rpu_scalar_mul_vec3_f64)
                    local.get $occ
                    (call $_rpu_vec3_mul_scalar_f64)
                    (call $_rpu_vec3_add_vec3_f64)
                    local.set $color_z
                    local.set $color_y
                    local.set $color_x
                    local.get $color_x
                    local.get $color_y
                    local.get $color_z
                    local.get $dif
                    local.get $spe
                    (f64.mul)
                    global.get $specularColor_x
                    global.get $specularColor_y
                    global.get $specularColor_z
                    (call $_rpu_scalar_mul_vec3_f64)
                    local.get $occ
                    (call $_rpu_vec3_mul_scalar_f64)
                    (call $_rpu_vec3_add_vec3_f64)
                    local.set $color_z
                    local.set $color_y
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
        local.set $c_z
        local.set $c_y
        local.set $c_x
        local.get $c_x
        local.get $c_y
        local.get $c_z
        (f64.const 1)
        (return)
    )
)
