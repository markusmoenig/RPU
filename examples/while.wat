(module

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
