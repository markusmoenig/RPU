(module

    (memory 1)

    (global $mem_ptr (export "mem_ptr") (mut i32) (i32.const 32)) ;; We keep the first 32 bytes to shuffle stack content

    ;; Allocate memory and move the memory ptr
    (func $malloc (param $size i32) (result i32)
      (local $current_ptr i32)
      (set_local $current_ptr (global.get $mem_ptr))
      (global.set $mem_ptr
        (i32.add (get_local $current_ptr) (get_local $size)))
      (get_local $current_ptr)
    )

    ;; function 'getter'
    (func $getter  (result i32)
        (local $ray i32)
        (local $_rpu_temp_f64 f64)
        (f64.const 1)
        (f64.const 2)
        (f64.const 3)
        (f64.const 0)
        (f64.const 1)
        (f64.const 0)
        (i32.const 48)
        (call $malloc)
        (local.set $ray)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 40
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 32
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 24
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 16
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 8
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 0
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        (f64.const 8)
        (f64.const 1)
        (f64.const 7)
        (local.set $_rpu_temp_f64)
        (local.get $ray)
        (i32.const 40)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $ray)
        (i32.const 32)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.set $_rpu_temp_f64)
        (local.get $ray)
        (i32.const 24)
        (i32.add)
        (local.get $_rpu_temp_f64)
        (f64.store)
        (local.get $ray)
        (return)
    )

    ;; function 'main'
    (func $main (export "main")  (result i64)
        (local $ray i32)
        (local $origin_x f64)
        (local $origin_y f64)
        (local $origin_z f64)
        (local $_rpu_temp_f64 f64)
        (local $a i64)
        (local $i i64)
        (call $getter)
        (local.set $ray)
        (f64.const 7)
        (f64.const 8)
        (f64.const 9)
        local.set $origin_z
        local.set $origin_y
        local.set $origin_x
        (f64.const 7)
        (f64.const 8)
        (f64.const 9)
        (f64.const 7)
        (f64.const 8)
        (f64.const 9)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 40
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 32
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 24
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 16
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 8
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        local.set $_rpu_temp_f64
        local.get $ray
        i32.const 0
        i32.add
        local.get $_rpu_temp_f64
        (f64.store)
        (i64.const 0)
        local.set $a

        (i64.const 0)
        local.set $i
        (block
            (loop
                local.get $i
                (i64.const 10)
                (i64.lt_s)
                (i32.eqz)
                (br_if 1)
                (block
                    (i64.const 1)
                    local.get $a
                    i64.add
                    local.set $a
                )
                (i64.const 1)
                local.get $i
                i64.add
                local.set $i
                (br 0)
            )
        )
        local.get $i
        (return)
    )
)
