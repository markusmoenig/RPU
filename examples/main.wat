(module
    (import "env" "_rpu_rand" (func $_rpu_rand (result f64)))

    (memory 1)

    ;; function 'main'
    (func $main (export "main")  (result f64)
        (local $result f64)
        (call $_rpu_rand)
        local.set $result
        local.get $result
        (return)
    )
)
