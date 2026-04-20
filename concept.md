The source reference for the original cross-platform app layer now lives outside this repo in `../DenrimNoise`. It runs natively on Windows / Linux / macOS / iPadOS via an app abstraction layer, native WGPU / winit window layer, and a macOS FFI which allows creating a static library, loading it into Xcode, and publishing.

Now read `goal.md`, which describes the RPU project goal. RPU uses the extracted runtime/app-layer ideas, but the in-repo implementation is now `rpu-scenevm` plus the `rpu-*` crates.

My questions:

* As RPU would be distributed as an Rust crates.io CLI project. The build functionality could create a temp project, embedd the project files and build a standalone app for all platforms on its own ? It could drive cargo right ? All the user would need would be to install Docker ?

* Would this also be possible for the Xcode project somehow ? We build it, we get an xcode project ready to go ?

* Obviously the intern run command would just open the winit window locally and render the project.
