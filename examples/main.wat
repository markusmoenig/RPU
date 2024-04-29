(module
    (memory 1)

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

    ;; function 'main'
    (func $main (export "main") (param $test f64) (result f64 f64 f64)
        (local $result_x f64)
        (local $result_y f64)
        (local $result_z f64)
        
        (f64.const 1)
        (f64.const 3)
        (f64.const 5)
        (call $_rpu_normalize_vec3_f64)
        local.set $result_z
        local.set $result_y
        local.set $result_x
        local.get $result_x
        local.get $result_y
        local.get $result_z
        
        (return)
    )
)
