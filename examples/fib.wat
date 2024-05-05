(module
    (import "env" "_rpu_sin" (func $_rpu_sin (param f64) (result f64)))
    (import "env" "_rpu_cos" (func $_rpu_cos (param f64) (result f64)))
    (import "env" "_rpu_tan" (func $_rpu_tan (param f64) (result f64)))
    (import "env" "_rpu_degrees" (func $_rpu_degrees (param f64) (result f64)))
    (import "env" "_rpu_radians" (func $_rpu_radians (param f64) (result f64)))
    (import "env" "_rpu_min" (func $_rpu_min (param f64) (param f64) (result f64)))
    (import "env" "_rpu_max" (func $_rpu_max (param f64) (param f64) (result f64)))
    (import "env" "_rpu_pow" (func $_rpu_pow (param f64) (param f64) (result f64)))

    (memory 1)

    ;; function 'fib'
    (func $fib (param $n i64) (result i64)

        local.get $n
        
        (i64.const 1)
        (i64.le_s)
        (if
            (then
                local.get $n
                
                (return)
            )
        )
        
        local.get $n
        
        (i64.const 2)
        (i64.sub)
        (call $fib)
        
        local.get $n
        
        (i64.const 1)
        (i64.sub)
        (call $fib)
        (i64.add)
        (return)
    )

    ;; function 'main'
    (func $main (export "main") (param $x i64) (result i64)
        
        local.get $x
        
        (call $fib)
        (return)
    )
)
