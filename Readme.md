RPU is a GLSL compatible programming language for rendering procedural graphics on the CPU.

As GPU shaders can limit the complexity of what you can render, RPU aims to provide an alternative way of rendering complex, unlimited procedural graphics on the CPU, in 64-bit or 32-bit precision.

RPU strives to be compabible with GLSL which means that you can easily port your existing shaders to RPU.

Alternatively you can also use RPU as a general purpose mathematical scripting language, as it is designed to be fast and embeddable in your applications.

## Features

- GLSL compatible
- 64-bit or 32-bit precision (decide on compile time)
- Unlimited procedural graphics
- Easy to port existing shaders
- Fast and embeddable in your applications
- Run shaders in your terminal via rpuc (see [Usage](./usage) for more info)

RPU compiles to WebAssembly (WAT) and uses [wasmer](https://github.com/wasmerio/wasmer) as a runtime. Which means RPU has near native speed, is hot-reloadable and can run on any platform that wasmer supports.

For shaders it uses a multi-threaded tiled rendering approach, which splits the image into tiles and renders each tile in parallel.

## Current Limitations

- Only signed integers are supported at the moment, i.e. no unsigned integer types and their associated bit operations. As RPU has a `rand()` function which generates high quality random numbers on the Rust side, I do not see unsigned integers as a priority right now.

- Function parameters do not support `in`, `out` or `inout` right now. Vectors and matrices are passed by value, structs are passed by reference. **I will add support for inout parameters in the near future.**

- No textures yet, coming soon.

- No preprocessor yet, coming soon.

## Goals

- Create a fast and embeddable GLSL compatible language for procedural graphics

- Create a module system to easily import noise libraries, renderers, cameras etc (TBD)

- Mesh generation for 3D SDF maps (TBD)

## Getting Started

Use the `export` keyword to export the function you want to run. For example to run a fibonacci sequence:

```glsl
int fib(int n) {
  if (n <= 1) return n;
  return fib(n - 2) + fib(n - 1);
}

export int main(int x) {
  return fib(x);
}
```

You could then run this with `rpuc --source fib.rpu -f main -a 42` to get the fibonacci sequence of 42 which takes around a second on my machine.

Shaders have a signature of

```glsl
export vec4 shader(vec2 coord, vec2 resolution) {
  return vec4(1); // For an all white image
}
```

You could run this via `rpuc --source myshader.rpu -f shader --write`.

The resulting image will be saved by _rpuc_ as `myshader.png`. The `--write` flag tells rpuc to write the image to disk every time a tile is completed. Giving a live preview of the rendering process.

RPU assumes that your shader uses stochastic sampling for anti-aliasing. You can pass the `--iterations` flag to _rpuc_ to specify the number of samples per pixel.

A simple raymarching example:

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
    // Generate the uv with jittering for anti-aliasing
    vec2 uv = (2.0 * (coord + vec2(rand(), rand())) - resolution.xy) / resolution.y;

    vec3 ro = vec3(.7, .8, -1.);
    vec3 rd = GetRayDir(uv, ro, vec3(0), 1.);

    float t = 0.;
    float max_t = 2.;

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
        t += d;
    }

    return col;
}
```

By executing the shader it generates the following image:
![Raymarch](examples/raymarch.png)

# Sponsors

None yet, but you can [sponsor me on GitHub](https://github.com/sponsors/markusmoenig/).
