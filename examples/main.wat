(module
    (import "env" "_rpu_fract" (func $_rpu_fract (param f64) (result f64)))

    (memory 1)

    ;; vec3 fract
    (func $_rpu_vec3_fract_f64  (param $x f64)  (param $y f64)  (param $z f64)  (result f64 f64 f64)
        local.get $x
        (call $_rpu_fract)
        local.get $y
        (call $_rpu_fract)
        local.get $z
        (call $_rpu_fract))

    ;; function 'main'
    (func $main (export "main")  (result f64 f64 f64)
        (local $result_x f64)
        (local $result_y f64)
        (local $result_z f64)
        (f64.const 3.32)
        (f64.const 4.3)
        (f64.const 4)
        (call $_rpu_vec3_fract_f64)
        local.set $result_z
        local.set $result_y
        local.set $result_x
        local.get $result_x
        local.get $result_y
        local.get $result_z
        (return)
    )
)
