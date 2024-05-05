RPU is a GLSL / C like programming language which compiles to WebAssembly (WAT) and is currently under development.

You can use it as a general purpose programming language, as a shader language for 2D and 3D renderering and as a (very) fast embedded scripting language for Rust based applications.

RPU compiles to WAT code and uses [wasmer](https://crates.io/crates/wasmer) as a runtime. The GLSL features like vecs, swizzles and math functions get compiled on-demand. They do not introduce any overhead or speed / size penalties if not used.

You can choose between 32 and 64 bit precision during compile time.

All vector based operations (length, dot, cross etc) are implemented in pure WebAssembly. Trigonometric functions are implemented in Rust and are called via the wasmer runtime.

## Currently implemented

- [x] Basic types: int, ivec2, ivec3, ivec4, float, vec2, vec3, vec4
- [x] Math operators: +, -, \*, /
- [x] Math functions: dot, cross, mix, smoothstep, length, normalize, sin, cos, sqrt, ceil, floor, abs, tan, degrees, radians.
- [x] Control structures: if, else, while, break, return, export
- [x] Swizzles: vec2.xy, vec3.xyz, vec4.xyzw etc

---

Fibonacci example like in a general purpose language:

```glsl
int fib(int n) {
  if (n <= 1) return n;
  return fib(n - 2) + fib(n - 1);
}

export int main(int x) {
  return fib(x);
}
```

You can see the generated WAT file [here](/examples/fib.wat).

To execute it with a secuence of 42 run `cargo run --release -- --source examples/fib.rpu -f main -a 42`. It executes in about a second on my M3.

---

A simple shader example:

```glsl
export vec4 shader(vec2 coord, vec2 resolution) {
    vec2 uv = (2.0 * coord - resolution.xy) / resolution.y;

    float d = length(uv) - 0.5;
    d = 1.0 - smoothstep(0.0, 0.01, d);

    vec4 c = mix(vec4(0.2, 0.2, 0.2, 1.0), vec4(1.0, 1.0, 1.0, 1.0), d);

    return c;
}
```

`cargo run --release -- --source examples/dic.rpu -f shader` generates the following image:
![Disc](/examples/disc.png)

This runs in about 90ms in 800x600 in 64-bit on my machine.
