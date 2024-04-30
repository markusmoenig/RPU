(module
    (memory 1)

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

    ;; function 'main'
    (func $main (export "main")  (result f64 f64 f64)
        (local $result_x f64)
        (local $result_y f64)
        (local $result_z f64)
        
        (f64.const 1)
        (f64.const 3)
        (f64.const 5)
        (f64.const 7)
        (f64.const 9)
        (f64.const 11)
        (call $_rpu_cross_product_f64)
        local.set $result_z
        local.set $result_y
        local.set $result_x
        local.get $result_x
        local.get $result_y
        local.get $result_z
        
        (return)
    )
)
