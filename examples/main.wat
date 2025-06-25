(module

    (memory 1)


    ;; function 'main'
    (func $main (export "main")  (result f64 f64 f64)
        (f64.const 0)
        (f64.const 1)
        (f64.const 2)
        (return)
    )
)
