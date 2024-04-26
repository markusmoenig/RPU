RPU is a GLSL like programming language which compiles to WebAssembly (WAT) and is currently under development.

You can use it as a general purpose programming language, as a shader language for 2D and 3D renderering and as a (very) fast embedded scripting language for Rust based applications.

RPU compiles to WAT code and uses wasmer as a runtime. The GLSL features like vecs, swizzles and math functions get compiled on-demand. They do not introduce any overhead or speed / size penalties if not used.
