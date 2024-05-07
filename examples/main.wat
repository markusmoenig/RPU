(module

    (memory 1)

    ;; function 'main'
    (func $main (export "main")  (result i64 i64)
        (local $test_x i64)
        (local $test_y i64)
        (i64.const 1)
        (i64.const 2)
        local.set $test_y
        local.set $test_x
        (i64.const 3)
        (i64.const 4)
        local.get $test_y
        i64.add
        local.set $test_y
        local.get $test_x
        i64.add
        local.set $test_x
        local.get $test_x
        local.get $test_y
        (return)
    )
)
