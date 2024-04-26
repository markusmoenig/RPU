RPU is a GLSL like programming language which compiles to WebAssembly (WAT) and is currently under development.

You can use it as a general purpose programming language, as a shader language for 2D and 3D renderering and as a (very) fast embedded scripting language for Rust based applications.

RPU compiles to WAT code and uses **wasmer** as a runtime. The GLSL features like vecs, swizzles and math functions get compiled on-demand. They do not introduce any overhead or speed / size penalties if not used.

Fibonacci example without any graphics features:

```glsl
int fib(int n) {
  if (n <= 1) return n;
  return fib(n - 2) + fib(n - 1);
}

export int main(int x) {
  return fib(x);
}
```

which gets compiled to:

```wasm
(module
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
```

A fibonacci sequence of 42 executes in about a second on my M3.

Something a bit more graphical:

```glsl
export ivec2 main(int x) {
    ivec2 result = 2 * ivec2(5, 2);

    return result.yx;
}
```

The GLSL features are right now under development.
