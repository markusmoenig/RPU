(module
    (memory 1)

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
        (f64.const 1)
        (f64.const 0)
        (f64.const 0)
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
