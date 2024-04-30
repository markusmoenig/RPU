(module
    (import "env" "_rpu_sin" (func $_rpu_sin (param f64) (result f64)))
    (import "env" "_rpu_cos" (func $_rpu_cos (param f64) (result f64)))

    (memory 1)

    ;; vec3 sin
    (func $_rpu_vec3_sin_f64  (param $x f64)  (param $y f64)  (param $z f64)  (result f64 f64 f64)
        local.get $x
        (call $_rpu_sin)
        local.get $y
        (call $_rpu_sin)
        local.get $z
        (call $_rpu_sin))

    ;; function 'main'
    (func $main (export "main")  (result f64 f64 f64)
        (local $result_x f64)
        (local $result_y f64)
        (local $result_z f64)
        
        (f64.const 1)
        (f64.const 3)
        (f64.const 4)
        (call $_rpu_vec3_sin_f64)
        local.set $result_z
        local.set $result_y
        local.set $result_x
        local.get $result_x
        local.get $result_y
        local.get $result_z
        
        (return)
    )
)
