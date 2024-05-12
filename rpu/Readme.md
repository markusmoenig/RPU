RPU is a GLSL compatible programming language for rendering procedural graphics on the CPU.

For more information visit [rpu-lang.org](https://rpu-lang.org/).

# Usage

To execute a function:

```rust
let fib = r#"
int fib(int n) {
  if (n <= 1) return n;
  return fib(n - 2) + fib(n - 1);
}

export int main(int x) {
    return fib(x);
}"#;

let rpu = RPU::new();
let use_64_bit = true;
if let Ok(rc) = rpu.compile_and_run(source, "main", [WasmValue::I64(10)], use_64_bit) {
    assert_eq!(
        rc, Ok(vec![WasmValue::I64(55)])
    );
}
```

If you only want to compile to WAT you can call:

```rust
let rc = compile_to_wat(source);
```

It returns a String containing the WAT source code.

To run the WAT source as a shader use

```rust
let mut buffer = ColorBuffer::new(800, 600);
let rc = rpu.compile_wat_and_run_as_shader(&wat, "shader", &mut buffer, use_64_bit);
```

The color buffer will contain the shader output. This runs the shader in a single thread. To run the shader in parallel use:

```rust
let mut buffer = Arc::new(Mutex::new(ColorBuffer::new(800, 600)));
let rc = rpu.compile_wat_and_run_as_tiled_shader(&wat, "shader", &mut buffer, (80, 80), 1, use_64_bit);
```

Where (80, 80) is the tile size. The buffer is wrapped in an Arc<Mutex<>> to allow multiple threads to write to it. The '1' is the number of iterations to compute (in case the shader is a path tracer).
