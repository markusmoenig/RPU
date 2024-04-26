(module
    (memory 1)

    ;; function 'main'
    (func $main (export "main") (param $x i64) (result i64)
        (local $result i64)
        (i64.const 2)
        (i64.const 2)
        (i64.mul)
        local.set $result
        local.get $result
        
        (return)
    )
)
