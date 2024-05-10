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
    (func $main (export "main")  (result f64 f64 f64)
        (local $ray i32)
        (call $getter)
        (local.set $ray)
        (local.get $ray)
        (i32.const 24)
        (i32.add)
        (f64.load)
        (local.get $ray)
        (i32.const 32)
        (i32.add)
        (f64.load)
        (local.get $ray)
        (i32.const 40)
        (i32.add)
        (f64.load)
        (return)
    )
)
