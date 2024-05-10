RPU is a GLSL / C like language for rendering procedural content without limitations. It is currently under development.

RPU compiles to WebAssembly (WAT) code and uses [wasmer](https://crates.io/crates/wasmer) as a runtime.

All vector based operations (length, dot, cross etc) are implemented in pure WebAssembly. Trigonometric functions are implemented in Rust and are called via the wasmer runtime.

You can choose between 32 and 64 bit precision during compile time.

When working on shaders, RPU uses multiple threads to render the image. This is done by splitting the image into tiles and rendering each tile in parallel.

# CPU vs GPU for procedural content

- CPU+: Unlimited procedural content in 64-bit (or 32-bit) precision. No need to worry that the shader will not compile.

- CPU+: Shader compile times are in milliseconds.

- CPU+: Access to host functionality like high-quality random numbers.

- CPU+: Recursive functions are supported.

- GPU+: Real-time rendering.

- CPU-: Slower rendering. However single-pass shaders of complex scenes render in a second or so (preview). Pathtracers can take minutes to render. The more complex the scene, the more the CPU catches up to the GPU.

# RPU differences to GLSL

- Only signed integers are supported at the moment, i.e. no unsigned integer types and their associated bit operations. As RPU has a `rand()` function which generates high quality random numbers on the Rust side, I do not see unsigned integers as a priority right now.

- Function parameters do not support `in`, `out` or `inout` right now. Vectors and matrices are passed by value, structs are passed by reference. **I will add support for inout parameters in the near future.**

- No preprocessor yet, coming soon.

# Currently implemented

- Basic types: int, ivec2, ivec3, ivec4, float, vec2, vec3, vec4, mat2, mat3, mat4 and custom structs
- Math operators: +, -, \*, /
- Math functions: dot, cross, mix, smoothstep, length, normalize, sin, cos, sqrt, ceil, floor, fract, abs, tan, degrees, radians, min, max, pow, rand, clamp
- Control structures: if, else, ternary (?:), while, break, return, const, export
- Assignment: =, +=, -=, \*=, /=
- Swizzles: vec2.xy, vec3.xyz, vec4.xyzw etc

# Planned Features

- A module system to import 2D / 3D renderers, noises, cameras etc.

- Integrated mesh generation.

- 100% GLSL compatibility over time.

- Better error messages.

# Usage

You can use the RPU compiler as a [standalone tool](https://crates.io/crates/rpuc) or as a [crate](https://crates.io/crates/rpu) in your Rust project.

# Examples

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

A simple shader example using raymarching:

```glsl
// Based on https://www.shadertoy.com/view/WtGXDD

float sdBox(vec3 p, vec3 s) {
    p = abs(p)-s;
	return length(max(p, 0.))+min(max(p.x, max(p.y, p.z)), 0.);
}

float GetDist(vec3 p) {
    float d = sdBox(p, vec3(.5));
    return d;
}

vec3 GetRayDir(vec2 uv, vec3 p, vec3 l, float z) {
    vec3 f = normalize(l-p);
    vec3 r = normalize(cross(vec3(0,1,0), f));
    vec3 u = cross(f,r);
    vec3 c = f*z;
    vec3 i = c + uv.x*r + uv.y*u;
    return normalize(i);
}

vec3 GetNormal(vec3 p) {
    vec2 e = vec2(0.001, 0.);
    vec3 n = GetDist(p) - vec3(GetDist(p-e.xyy), GetDist(p-e.yxy), GetDist(p-e.yyx));
    return normalize(n);
}

export vec4 shader(vec2 coord, vec2 resolution) {
    // Generate the uv with random jittering for anti-aliasing
    vec2 uv = (2.0 * (coord + vec2(rand(), rand())) - resolution.xy) / resolution.y;

    vec3 ro = vec3(.7, .8, -1.);
    vec3 rd = GetRayDir(uv, ro, vec3(0), 1.);

    float t = 0.;
    float max_t = 10.;

    vec4 col = vec4(uv.x, uv.y, 0., 1.);

    while (t < max_t) {
        vec3 p = ro + rd * t;
        float d = GetDist(p);
        if (abs(d) < 0.001) {
            vec3 n = GetNormal(p);
            float dif = dot(n, normalize(vec3(1, 2, 3))) * 0.5 + 0.5;
            col.xyz = pow(vec3(dif), .4545);

            break;
        }
        t = t + d;
    }

    return col;
}
```

By executing the shader it generates the following image:
![Raymarch](examples/raymarch.png)

This runs in about 150ms in 800x600 in 64-bit on my machine. You can run the example with `cargo run --release -- --source examples/raymarch.rpu -f shader`.
