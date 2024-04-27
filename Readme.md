RPU is a GLSL / C like programming language which compiles to WebAssembly (WAT) and is currently under development.

You can use it as a general purpose programming language, as a shader language for 2D and 3D renderering and as a (very) fast embedded scripting language for Rust based applications.

RPU compiles to WAT code and uses **wasmer** as a runtime. The GLSL features like vecs, swizzles and math functions get compiled on-demand. They do not introduce any overhead or speed / size penalties if not used.

You can choose between 32 and 64 bit precision during compile time.

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

You can see the generated WAT file [here](/examples/fib.wat).

A fibonacci sequence of 42 executes in about a second on my M3.

Something a bit more graphical:

```glsl
export ivec2 main(int x) {
    ivec2 result = 2 * ivec2(5, 2);

    return result.yx;
}
```

The GLSL features are right now under development.
