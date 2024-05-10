(module
    (import "env" "_rpu_sin" (func $_rpu_sin (param f64) (result f64)))
    (import "env" "_rpu_cos" (func $_rpu_cos (param f64) (result f64)))
    (import "env" "_rpu_tan" (func $_rpu_tan (param f64) (result f64)))
    (import "env" "_rpu_min" (func $_rpu_min (param f64) (param f64) (result f64)))
    (import "env" "_rpu_max" (func $_rpu_max (param f64) (param f64) (result f64)))
    (import "env" "_rpu_rand" (func $_rpu_rand (result f64)))

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

    ;; vec3 neg
    (func $_rpu_vec3_neg_f64  (param $x f64)  (param $y f64)  (param $z f64)  (result f64 f64 f64)
        local.get $x
        f64.neg
        local.get $y
        f64.neg
        local.get $z
        f64.neg)

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

    ;; vec1 sqrt
    (func $_rpu_vec1_sqrt_f64  (param $x f64)  (result f64)
        local.get $x
        f64.sqrt)

    ;; vec1 sin
    (func $_rpu_vec1_sin_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_sin))

    ;; vec1 cos
    (func $_rpu_vec1_cos_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_cos))

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

    ;; vec1 tan
    (func $_rpu_vec1_tan_f64  (param $x f64)  (result f64)
        local.get $x
        (call $_rpu_tan))

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

    ;; vec1 abs
    (func $_rpu_vec1_abs_f64  (param $x f64)  (result f64)
        local.get $x
        f64.abs)

    ;; function 'opU'
    (func $opU (param $o1_x f64) (param $o1_y f64)(param $o2_x f64) (param $o2_y f64) (result f64 f64)
        (local $_rpu_ternary_0_x f64)
        (local $_rpu_ternary_0_y f64)

        local.get $o1_x
        local.get $o2_x
        (f64.lt)
        (if
            (then
                local.get $o1_x
                local.get $o1_y
                (local.set $_rpu_ternary_0_y)
                (local.set $_rpu_ternary_0_x)
            )
            (else
                local.get $o2_x
                local.get $o2_y
                (local.set $_rpu_ternary_0_y)
                (local.set $_rpu_ternary_0_x)
            )
        )
        (local.get $_rpu_ternary_0_x)
        (local.get $_rpu_ternary_0_y)
        (return)
    )

    ;; function 'sdBox'
    (func $sdBox (param $p_x f64) (param $p_y f64) (param $p_z f64)(param $s_x f64) (param $s_y f64) (param $s_z f64) (result f64)
        local.get $p_x
        local.get $p_y
        local.get $p_z
        (call $_rpu_vec3_abs_f64)
        local.get $s_x
        local.get $s_y
        local.get $s_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $p_z
        local.set $p_y
        local.set $p_x
        local.get $p_x
        local.get $p_y
        local.get $p_z
        (f64.const 0)
        (call $_rpu_vec3_max_f64)
        (call $_rpu_vec3_length_f64)
        local.get $p_x
        local.get $p_y
        local.get $p_z
        (call $_rpu_vec1_max_f64)
        (call $_rpu_vec1_max_f64)
        (f64.const 0)
        (call $_rpu_vec1_min_f64)
        (f64.add)
        (return)
    )

    ;; function 'sdPlane'
    (func $sdPlane (param $p_x f64) (param $p_y f64) (param $p_z f64)(param $n_x f64) (param $n_y f64) (param $n_z f64) (param $n_w f64) (result f64)
        local.get $p_x
        local.get $p_y
        local.get $p_z
        local.get $n_x
        local.get $n_y
        local.get $n_z
        (call $_rpu_dot_product_vec3_f64)
        local.get $n_w
        (f64.add)
        (return)
    )

    ;; function 'GetDist'
    (func $GetDist (param $p_x f64) (param $p_y f64) (param $p_z f64) (result f64 f64)
        (local $box_x f64)
        (local $box_y f64)
        (local $plane_x f64)
        (local $plane_y f64)
        local.get $p_x
        local.get $p_y
        local.get $p_z
        (f64.const 0)
        (f64.const 0.5)
        (f64.const 0)
        (call $_rpu_vec3_neg_f64)
        (call $_rpu_vec3_add_vec3_f64)
        (f64.const 0.5)
        (f64.const 0.5)
        (f64.const 0.5)
        (call $sdBox)
        (f64.const 0)
        local.set $box_y
        local.set $box_x
        local.get $p_x
        local.get $p_y
        local.get $p_z
        (f64.const 0)
        (f64.const 1)
        (f64.const 0)
        (f64.const 0)
        (call $sdPlane)
        (f64.const 1)
        local.set $plane_y
        local.set $plane_x
        local.get $box_x
        local.get $box_y
        local.get $plane_x
        local.get $plane_y
        (call $opU)
        (return)
    )

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

    ;; function 'GetNormal'
    (func $GetNormal (param $p_x f64) (param $p_y f64) (param $p_z f64) (result f64 f64 f64)
        (local $e_x f64)
        (local $e_y f64)
        (local $_rpu_temp_f64 f64)
        (local $n_x f64)
        (local $n_y f64)
        (local $n_z f64)
        (f64.const 0.001)
        (f64.const 0)
        local.set $e_y
        local.set $e_x
        local.get $p_x
        local.get $p_y
        local.get $p_z
        (call $GetDist)
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
        local.get $p_x
        local.get $p_y
        local.get $p_z
        local.get $e_x
        local.get $e_y
        local.get $e_y
        (call $_rpu_vec3_sub_vec3_f64)
        (call $GetDist)
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
        local.get $p_x
        local.get $p_y
        local.get $p_z
        local.get $e_y
        local.get $e_x
        local.get $e_y
        (call $_rpu_vec3_sub_vec3_f64)
        (call $GetDist)
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
        local.get $p_x
        local.get $p_y
        local.get $p_z
        local.get $e_y
        local.get $e_y
        local.get $e_x
        (call $_rpu_vec3_sub_vec3_f64)
        (call $GetDist)
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
        local.set $n_z
        local.set $n_y
        local.set $n_x
        local.get $n_x
        local.get $n_y
        local.get $n_z
        (call $_rpu_normalize_vec3_f64)
        (return)
    )

    ;; function 'RandomInUnitDisk'
    (func $RandomInUnitDisk  (result f64 f64)
        (local $h_x f64)
        (local $h_y f64)
        (local $phi f64)
        (local $r f64)
        (call $_rpu_rand)
        (call $_rpu_rand)
        (f64.const 1)
        (f64.const 6.2831855)
        (call $_rpu_vec2_mul_vec2_f64)
        local.set $h_y
        local.set $h_x
        local.get $h_y
        local.set $phi
        local.get $h_x
        (call $_rpu_vec1_sqrt_f64)
        local.set $r
        local.get $r
        local.get $phi
        (call $_rpu_vec1_sin_f64)
        local.get $phi
        (call $_rpu_vec1_cos_f64)
        (call $_rpu_scalar_mul_vec2_f64)
        (return)
    )

    ;; function 'CreateRay'
    (func $CreateRay (param $uv_x f64) (param $uv_y f64)(param $resolution_x f64) (param $resolution_y f64)(param $origin_x f64) (param $origin_y f64) (param $origin_z f64)(param $lookAt_x f64) (param $lookAt_y f64) (param $lookAt_z f64) (result i32)
        (local $ray i32)
        (local $_rpu_temp_f64 f64)
        (local $aperture f64)
        (local $focus_dist f64)
        (local $fov f64)
        (local $lens_radius f64)
        (local $theta f64)
        (local $half_height f64)
        (local $half_width f64)
        (local $w_x f64)
        (local $w_y f64)
        (local $w_z f64)
        (local $u_x f64)
        (local $u_y f64)
        (local $u_z f64)
        (local $v_x f64)
        (local $v_y f64)
        (local $v_z f64)
        (local $lower_left_corner_x f64)
        (local $lower_left_corner_y f64)
        (local $lower_left_corner_z f64)
        (local $horizontal_x f64)
        (local $horizontal_y f64)
        (local $horizontal_z f64)
        (local $vertical_x f64)
        (local $vertical_y f64)
        (local $vertical_z f64)
        (local $unit_x f64)
        (local $unit_y f64)
        (local $offset_x f64)
        (local $offset_y f64)
        (local $offset_z f64)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (f64.const 0)
        (i32.const 48)
        (call $malloc)
        (local.set $ray)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 40
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 32
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 24
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 16
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 8
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 0
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        (f64.const 0.1)
        local.set $aperture
        (f64.const 10)
        local.set $focus_dist
        (f64.const 40)
        local.set $fov
        local.get $aperture
        (f64.const 2)
        (f64.div)
        local.set $lens_radius
        local.get $fov
        (f64.const 3.1415927)
        (f64.mul)
        (f64.const 180)
        (f64.div)
        local.set $theta
        local.get $theta
        (f64.const 2)
        (f64.div)
        (call $_rpu_vec1_tan_f64)
        local.set $half_height
        local.get $resolution_x
        local.get $resolution_y
        (f64.div)
        local.get $half_height
        (f64.mul)
        local.set $half_width
        local.get $origin_x
        local.get $origin_y
        local.get $origin_z
        local.get $lookAt_x
        local.get $lookAt_y
        local.get $lookAt_z
        (call $_rpu_vec3_sub_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $w_z
        local.set $w_y
        local.set $w_x
        (f64.const 0)
        (f64.const 1)
        (f64.const 0)
        local.get $w_x
        local.get $w_y
        local.get $w_z
        (call $_rpu_cross_product_f64)
        (call $_rpu_normalize_vec3_f64)
        local.set $u_z
        local.set $u_y
        local.set $u_x
        local.get $w_x
        local.get $w_y
        local.get $w_z
        local.get $u_x
        local.get $u_y
        local.get $u_z
        (call $_rpu_cross_product_f64)
        local.set $v_z
        local.set $v_y
        local.set $v_x
        local.get $origin_x
        local.get $origin_y
        local.get $origin_z
        local.get $half_width
        local.get $focus_dist
        (f64.mul)
        local.get $u_x
        local.get $u_y
        local.get $u_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_sub_vec3_f64)
        local.get $half_height
        local.get $focus_dist
        (f64.mul)
        local.get $v_x
        local.get $v_y
        local.get $v_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_sub_vec3_f64)
        local.get $w_x
        local.get $w_y
        local.get $w_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.set $lower_left_corner_z
        local.set $lower_left_corner_y
        local.set $lower_left_corner_x
        (f64.const 2)
        local.get $half_width
        (f64.mul)
        local.get $focus_dist
        (f64.mul)
        local.get $u_x
        local.get $u_y
        local.get $u_z
        (call $_rpu_scalar_mul_vec3_f64)
        local.set $horizontal_z
        local.set $horizontal_y
        local.set $horizontal_x
        (f64.const 2)
        local.get $half_height
        (f64.mul)
        local.get $focus_dist
        (f64.mul)
        local.get $v_x
        local.get $v_y
        local.get $v_z
        (call $_rpu_scalar_mul_vec3_f64)
        local.set $vertical_z
        local.set $vertical_y
        local.set $vertical_x
        local.get $lens_radius
        (call $RandomInUnitDisk)
        (call $_rpu_scalar_mul_vec2_f64)
        local.set $unit_y
        local.set $unit_x
        local.get $u_x
        local.get $u_y
        local.get $u_z
        local.get $unit_x
        (call $_rpu_vec3_mul_scalar_f64)
        local.get $v_x
        local.get $v_y
        local.get $v_z
        local.get $unit_y
        (call $_rpu_vec3_mul_scalar_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.set $offset_z
        local.set $offset_y
        local.set $offset_x
        local.get $origin_x
        local.get $origin_y
        local.get $origin_z
        local.get $offset_x
        local.get $offset_y
        local.get $offset_z
        (call $_rpu_vec3_add_vec3_f64)
        (local.set $_rpu_temp_f64)
        (local.get $ray)
        (i32.const 16)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $ray)
        (i32.const 8)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $ray)
        (i32.const 0)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        local.get $lower_left_corner_x
        local.get $lower_left_corner_y
        local.get $lower_left_corner_z
        local.get $uv_x
        local.get $horizontal_x
        local.get $horizontal_y
        local.get $horizontal_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $uv_y
        local.get $vertical_x
        local.get $vertical_y
        local.get $vertical_z
        (call $_rpu_scalar_mul_vec3_f64)
        (call $_rpu_vec3_add_vec3_f64)
        local.get $origin_x
        local.get $origin_y
        local.get $origin_z
        (call $_rpu_vec3_sub_vec3_f64)
        local.get $offset_x
        local.get $offset_y
        local.get $offset_z
        (call $_rpu_vec3_sub_vec3_f64)
        (call $_rpu_normalize_vec3_f64)
        (local.set $_rpu_temp_f64)
        (local.get $ray)
        (i32.const 40)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $ray)
        (i32.const 32)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $ray)
        (i32.const 24)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.get $ray)
        (return)
    )

    ;; function 'shader'
    (func $shader (export "shader") (param $coord_x f64) (param $coord_y f64)(param $resolution_x f64) (param $resolution_y f64) (result f64 f64 f64 f64)
        (local $uv_x f64)
        (local $uv_y f64)
        (local $origin_x f64)
        (local $origin_y f64)
        (local $origin_z f64)
        (local $lookAt_x f64)
        (local $lookAt_y f64)
        (local $lookAt_z f64)
        (local $ray i32)
        (local $t f64)
        (local $max_t f64)
        (local $col_x f64)
        (local $col_y f64)
        (local $col_z f64)
        (local $col_w f64)
        (local $p_x f64)
        (local $p_y f64)
        (local $p_z f64)
        (local $rc_x f64)
        (local $rc_y f64)
        (local $n_x f64)
        (local $n_y f64)
        (local $n_z f64)
        local.get $coord_x
        local.get $coord_y
        (call $_rpu_rand)
        (call $_rpu_rand)
        (call $_rpu_vec2_add_vec2_f64)
        local.get $resolution_x
        local.get $resolution_y
        (call $_rpu_vec2_div_vec2_f64)
        local.set $uv_y
        local.set $uv_x
        (f64.const 0)
        (f64.const 0.8)
        (f64.const 1)
        local.set $origin_z
        local.set $origin_y
        local.set $origin_x
        (f64.const 0)
        (f64.const 0.5)
        (f64.const 0)
        local.set $lookAt_z
        local.set $lookAt_y
        local.set $lookAt_x
        local.get $uv_x
        local.get $uv_y
        local.get $resolution_x
        local.get $resolution_y
        local.get $origin_x
        local.get $origin_y
        local.get $origin_z
        local.get $lookAt_x
        local.get $lookAt_y
        local.get $lookAt_z
        (call $CreateRay)
        (local.set $ray)
        (f64.const 0)
        local.set $t
        (f64.const 100)
        local.set $max_t
        local.get $uv_x
        local.get $uv_y
        (f64.const 0)
        (f64.const 1)
        local.set $col_w
        local.set $col_z
        local.set $col_y
        local.set $col_x

        (block
            (loop
                local.get $t
                local.get $max_t
                (f64.lt)
                (i32.eqz)
                (br_if 1)
                (block
                    (local.get $ray)
                    (i32.const 0)
                    (i32.add)
                    (f64.load)
                    (local.get $ray)
                    (i32.const 8)
                    (i32.add)
                    (f64.load)
                    (local.get $ray)
                    (i32.const 16)
                    (i32.add)
                    (f64.load)
                    (local.get $ray)
                    (i32.const 24)
                    (i32.add)
                    (f64.load)
                    (local.get $ray)
                    (i32.const 32)
                    (i32.add)
                    (f64.load)
                    (local.get $ray)
                    (i32.const 40)
                    (i32.add)
                    (f64.load)
                    local.get $t
                    (call $_rpu_vec3_mul_scalar_f64)
                    (call $_rpu_vec3_add_vec3_f64)
                    local.set $p_z
                    local.set $p_y
                    local.set $p_x
                    local.get $p_x
                    local.get $p_y
                    local.get $p_z
                    (call $GetDist)
                    local.set $rc_y
                    local.set $rc_x

                    local.get $rc_x
                    (call $_rpu_vec1_abs_f64)
                    (f64.const 0.001)
                    (f64.lt)
                    (if
                        (then
                            (block
                                local.get $p_x
                                local.get $p_y
                                local.get $p_z
                                (call $GetNormal)
                                local.set $n_z
                                local.set $n_y
                                local.set $n_x

                                local.get $rc_y
                                (f64.const 1)
                                (f64.eq)
                                (if
                                    (then
                                        (block
                                            (f64.const 1)
                                            (f64.const 0)
                                            (f64.const 0)
                                            (f64.const 1)
                                            local.set $col_w
                                            local.set $col_z
                                            local.set $col_y
                                            local.set $col_x
                                        )
                                    )
                                    (else
                                        (block
                                            (f64.const 0)
                                            (f64.const 1)
                                            (f64.const 0)
                                            (f64.const 1)
                                            local.set $col_w
                                            local.set $col_z
                                            local.set $col_y
                                            local.set $col_x
                                        )
                                    )
                                )
                                (br 4)
                            )
                        )
                    )
                    local.get $rc_x
                    local.get $t
                    f64.add
                    local.set $t
                )
                (br 0)
            )
        )
        local.get $col_x
        local.get $col_y
        local.get $col_z
        local.get $col_w
        (return)
    )
)
