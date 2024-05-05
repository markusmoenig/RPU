(module
    (import "env" "_rpu_sin" (func $_rpu_sin (param f64) (result f64)))
    (import "env" "_rpu_cos" (func $_rpu_cos (param f64) (result f64)))
    (import "env" "_rpu_tan" (func $_rpu_tan (param f64) (result f64)))
    (import "env" "_rpu_degrees" (func $_rpu_degrees (param f64) (result f64)))
    (import "env" "_rpu_radians" (func $_rpu_radians (param f64) (result f64)))

    (memory 1)

    ;; function 'main'
    (func $main (export "main")  (result i64)
        (local $counter i64)
        (i64.const 0)
        local.set $counter

        (block
            (loop
                local.get $counter
                
                (i64.const 10)
                (i64.lt_s)
                (i32.eqz)
                (br_if 1)
                (block
                    local.get $counter
                    
                    (i64.const 1)
                    (i64.add)
                    local.set $counter

                    local.get $counter
                    
                    (i64.const 5)
                    (i64.eq)
                    (if
                        (then
                            (block
                                (br 4)
                            )
                        )
                    )
                )
                (br 0)
            )
        )
        local.get $counter
        
        (return)
    )
)
