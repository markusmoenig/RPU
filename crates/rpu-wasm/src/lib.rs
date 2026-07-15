use anyhow::{Context, Result, bail};
use rpu_core::{CapabilityConfig, wasm_abi};
use rpu_graphics::GraphicsService;
use rpu_system::SystemService;
use wasmer::{Imports, Instance, Memory, Module, RuntimeError, Store, TypedFunction};

pub fn run_cli(bytes: &[u8], requires: &CapabilityConfig, args: &[String]) -> Result<i32> {
    let mut guest = instantiate(bytes, requires, args, "cartridge")?;
    validate_cli_export(&mut guest.store, &guest.instance)?;

    let run: TypedFunction<(), i32> = guest
        .instance
        .exports
        .get_typed_function(&mut guest.store, wasm_abi::EXPORT_RUN)
        .context("invalid `rpu_run` export")?;
    match run.call(&mut guest.store) {
        Ok(code) => Ok(code),
        Err(error) => match guest.services.exit_code(&error) {
            Some(code) => Ok(code),
            None => Err(error).context("`rpu_run` trapped"),
        },
    }
}

pub struct LoadedModule {
    _guest: WasmGuest,
}

pub fn load_module(
    bytes: &[u8],
    requires: &CapabilityConfig,
    args: &[String],
) -> Result<LoadedModule> {
    let mut guest = instantiate(bytes, requires, args, "module")?;
    let init: TypedFunction<(), i32> = guest
        .instance
        .exports
        .get_typed_function(&mut guest.store, wasm_abi::EXPORT_MODULE_INIT)
        .context("WASM module must export `rpu_module_init() -> i32`")?;
    let status = init
        .call(&mut guest.store)
        .context("`rpu_module_init` trapped")?;
    if status != 0 {
        bail!("`rpu_module_init` returned status {status}");
    }
    Ok(LoadedModule { _guest: guest })
}

struct WasmGuest {
    store: Store,
    instance: Instance,
    services: ServiceRegistry,
}

fn instantiate(
    bytes: &[u8],
    requires: &CapabilityConfig,
    args: &[String],
    guest_kind: &str,
) -> Result<WasmGuest> {
    let mut store = Store::default();
    let module = Module::new(&store, bytes)
        .with_context(|| format!("failed to compile WASM {guest_kind}"))?;
    let mut services = ServiceRegistry::new(&mut store, requires, args);
    services.validate_imports(&module, guest_kind)?;

    let instance = Instance::new(&mut store, &module, services.imports())
        .with_context(|| format!("failed to instantiate WASM {guest_kind}"))?;
    validate_common_exports(&mut store, &instance, guest_kind)?;

    let memory = instance
        .exports
        .get_memory(wasm_abi::MEMORY_EXPORT)
        .with_context(|| format!("WASM {guest_kind} must export `memory`"))?
        .clone();
    services.attach_memory(&mut store, memory);

    let abi_version: TypedFunction<(), i32> = instance
        .exports
        .get_typed_function(&mut store, wasm_abi::EXPORT_ABI_VERSION)
        .context("invalid `rpu_abi_version` export")?;
    let version = abi_version
        .call(&mut store)
        .context("`rpu_abi_version` trapped")?;
    if version != wasm_abi::ABI_VERSION as i32 {
        bail!(
            "unsupported RPU WASM ABI version {version}; host supports {}",
            wasm_abi::ABI_VERSION
        );
    }

    Ok(WasmGuest {
        store,
        instance,
        services,
    })
}

struct ServiceRegistry {
    imports: Imports,
    system: Option<SystemService>,
    graphics: Option<GraphicsService>,
}

impl ServiceRegistry {
    fn new(store: &mut Store, requires: &CapabilityConfig, args: &[String]) -> Self {
        let mut imports = Imports::new();
        let system = requires.system.then(|| {
            let service = SystemService::new(store, args);
            service.register(store, &mut imports);
            service
        });
        let graphics = requires.graphics.then(|| {
            let service = GraphicsService::new(store);
            service.register(store, &mut imports);
            service
        });
        Self {
            imports,
            system,
            graphics,
        }
    }

    fn imports(&self) -> &Imports {
        &self.imports
    }

    fn validate_imports(&self, module: &Module, guest_kind: &str) -> Result<()> {
        for import in module.imports() {
            if self.supports_import(import.module(), import.name()) {
                continue;
            }
            if import.module() == SystemService::namespace() {
                if self.system.is_none() {
                    bail!(
                        "WASM {guest_kind} imports `{}.{}` but `requires.system` is false",
                        import.module(),
                        import.name()
                    );
                }
                bail!(
                    "unsupported WASM system import `{}.{}`",
                    import.module(),
                    import.name()
                );
            }
            if import.module() == GraphicsService::namespace() {
                if self.graphics.is_none() {
                    bail!(
                        "WASM {guest_kind} imports `{}.{}` but `requires.graphics` is false",
                        import.module(),
                        import.name()
                    );
                }
                bail!(
                    "unsupported WASM graphics import `{}.{}`",
                    import.module(),
                    import.name()
                );
            }
            bail!(
                "unsupported WASM import `{}.{}`",
                import.module(),
                import.name()
            );
        }
        Ok(())
    }

    fn supports_import(&self, namespace: &str, name: &str) -> bool {
        (namespace == SystemService::namespace()
            && self.system.is_some()
            && SystemService::supports_import(name))
            || (namespace == GraphicsService::namespace()
                && self.graphics.is_some()
                && GraphicsService::supports_import(name))
    }

    fn attach_memory(&mut self, store: &mut Store, memory: Memory) {
        if let Some(system) = &self.system {
            system.attach_memory(store, memory);
        }
    }

    fn exit_code(&self, error: &RuntimeError) -> Option<i32> {
        self.system.as_ref()?;
        SystemService::exit_code(error)
    }

    #[cfg(test)]
    fn take_completed_graphics_frame(&self, store: &mut Store) -> Option<rpu_graphics::SceneFrame> {
        self.graphics.as_ref()?.take_completed_frame(store)
    }
}

fn validate_common_exports(store: &mut Store, instance: &Instance, guest_kind: &str) -> Result<()> {
    let _: TypedFunction<(), i32> = instance
        .exports
        .get_typed_function(store, wasm_abi::EXPORT_ABI_VERSION)
        .with_context(|| format!("WASM {guest_kind} must export `rpu_abi_version() -> i32`"))?;
    let _: TypedFunction<(i32, i32), i32> = instance
        .exports
        .get_typed_function(store, wasm_abi::EXPORT_ALLOC)
        .with_context(|| format!("WASM {guest_kind} must export `rpu_alloc(i32, i32) -> i32`"))?;
    let _: TypedFunction<(i32, i32, i32), ()> = instance
        .exports
        .get_typed_function(store, wasm_abi::EXPORT_DEALLOC)
        .with_context(|| format!("WASM {guest_kind} must export `rpu_dealloc(i32, i32, i32)`"))?;
    instance
        .exports
        .get_memory(wasm_abi::MEMORY_EXPORT)
        .with_context(|| format!("WASM {guest_kind} must export `memory`"))?;
    Ok(())
}

fn validate_cli_export(store: &mut Store, instance: &Instance) -> Result<()> {
    let _: TypedFunction<(), i32> = instance
        .exports
        .get_typed_function(store, wasm_abi::EXPORT_RUN)
        .context("WASM CLI cartridge must export `rpu_run() -> i32`")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_cli_module_with_arguments() {
        let wasm = br#"
            (module
                (import "rpu_system" "arg_count" (func $arg_count (result i32)))
                (memory (export "memory") 1)
                (func (export "rpu_abi_version") (result i32) i32.const 1)
                (func (export "rpu_alloc") (param i32 i32) (result i32) i32.const 1024)
                (func (export "rpu_dealloc") (param i32 i32 i32))
                (func (export "rpu_run") (result i32) call $arg_count)
            )
        "#;
        let requires = CapabilityConfig {
            system: true,
            graphics: false,
            audio: false,
            network: false,
        };
        let args = vec!["one".to_string(), "two".to_string()];

        assert_eq!(run_cli(wasm, &requires, &args).unwrap(), 2);
    }

    #[test]
    fn rejects_system_import_without_capability() {
        let wasm = br#"
            (module
                (import "rpu_system" "arg_count" (func (result i32)))
                (memory (export "memory") 1)
                (func (export "rpu_abi_version") (result i32) i32.const 1)
                (func (export "rpu_alloc") (param i32 i32) (result i32) i32.const 0)
                (func (export "rpu_dealloc") (param i32 i32 i32))
                (func (export "rpu_run") (result i32) i32.const 0)
            )
        "#;
        let requires = CapabilityConfig {
            system: false,
            graphics: false,
            audio: false,
            network: false,
        };

        let error = run_cli(wasm, &requires, &[]).unwrap_err().to_string();
        assert!(error.contains("requires.system"));
    }

    #[test]
    fn rejects_an_unregistered_service_namespace() {
        let wasm = br#"
            (module
                (import "rpu_audio" "play" (func))
                (memory (export "memory") 1)
                (func (export "rpu_abi_version") (result i32) i32.const 1)
                (func (export "rpu_alloc") (param i32 i32) (result i32) i32.const 0)
                (func (export "rpu_dealloc") (param i32 i32 i32))
                (func (export "rpu_run") (result i32) i32.const 0)
            )
        "#;
        let requires = CapabilityConfig {
            system: false,
            graphics: false,
            audio: false,
            network: false,
        };

        let error = run_cli(wasm, &requires, &[]).unwrap_err().to_string();
        assert!(error.contains("unsupported WASM import `rpu_audio.play`"));
    }

    #[test]
    fn rejects_graphics_import_without_capability() {
        let wasm = br#"
            (module
                (import "rpu_graphics" "begin_frame" (func (param i32 i32)))
                (memory (export "memory") 1)
                (func (export "rpu_abi_version") (result i32) i32.const 1)
                (func (export "rpu_alloc") (param i32 i32) (result i32) i32.const 0)
                (func (export "rpu_dealloc") (param i32 i32 i32))
                (func (export "rpu_run") (result i32) i32.const 0)
            )
        "#;
        let requires = CapabilityConfig {
            system: false,
            graphics: false,
            audio: false,
            network: false,
        };

        let error = run_cli(wasm, &requires, &[]).unwrap_err().to_string();
        assert!(error.contains("requires.graphics"));
    }

    #[test]
    fn rejects_an_unknown_import_from_the_graphics_service() {
        let wasm = br#"
            (module
                (import "rpu_graphics" "create_buffer" (func))
                (memory (export "memory") 1)
                (func (export "rpu_abi_version") (result i32) i32.const 1)
                (func (export "rpu_alloc") (param i32 i32) (result i32) i32.const 0)
                (func (export "rpu_dealloc") (param i32 i32 i32))
                (func (export "rpu_run") (result i32) i32.const 0)
            )
        "#;
        let requires = CapabilityConfig {
            system: false,
            graphics: true,
            audio: false,
            network: false,
        };

        let error = run_cli(wasm, &requires, &[]).unwrap_err().to_string();
        assert!(error.contains("unsupported WASM graphics import `rpu_graphics.create_buffer`"));
    }

    #[test]
    fn graphics_imports_build_a_portable_frame() {
        let wasm = br#"
            (module
                (import "rpu_graphics" "begin_frame" (func $begin_frame (param i32 i32)))
                (import "rpu_graphics" "clear" (func $clear (param f32 f32 f32 f32)))
                (import "rpu_graphics" "draw_rect" (func $draw_rect (param f32 f32 f32 f32 f32 f32 f32 f32)))
                (import "rpu_graphics" "end_frame" (func $end_frame))
                (memory (export "memory") 1)
                (func (export "rpu_abi_version") (result i32) i32.const 1)
                (func (export "rpu_alloc") (param i32 i32) (result i32) i32.const 0)
                (func (export "rpu_dealloc") (param i32 i32 i32))
                (func (export "rpu_run") (result i32)
                    i32.const 320
                    i32.const 180
                    call $begin_frame
                    f32.const 0.1
                    f32.const 0.2
                    f32.const 0.3
                    f32.const 1.0
                    call $clear
                    f32.const 12.0
                    f32.const 18.0
                    f32.const 64.0
                    f32.const 32.0
                    f32.const 1.0
                    f32.const 0.5
                    f32.const 0.25
                    f32.const 1.0
                    call $draw_rect
                    call $end_frame
                    i32.const 0)
            )
        "#;
        let requires = CapabilityConfig {
            system: false,
            graphics: true,
            audio: false,
            network: false,
        };
        let mut guest = instantiate(wasm, &requires, &[], "test cartridge").unwrap();
        validate_cli_export(&mut guest.store, &guest.instance).unwrap();
        let run: TypedFunction<(), i32> = guest
            .instance
            .exports
            .get_typed_function(&mut guest.store, wasm_abi::EXPORT_RUN)
            .unwrap();

        assert_eq!(run.call(&mut guest.store).unwrap(), 0);
        let frame = guest
            .services
            .take_completed_graphics_frame(&mut guest.store)
            .expect("graphics service completed a frame");
        assert_eq!(frame.size(), (320, 180));
        assert_eq!(frame.clear_color, [0.1, 0.2, 0.3, 1.0]);
        assert_eq!(frame.items.len(), 1);
        let rpu_graphics::RenderItem::Rect(rect) = &frame.items[0] else {
            panic!("expected a rectangle command");
        };
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (12.0, 18.0, 64.0, 32.0)
        );
        assert_eq!(rect.color, [1.0, 0.5, 0.25, 1.0]);
    }

    #[test]
    fn rejects_an_unknown_import_from_an_enabled_service() {
        let wasm = br#"
            (module
                (import "rpu_system" "unknown" (func))
                (memory (export "memory") 1)
                (func (export "rpu_abi_version") (result i32) i32.const 1)
                (func (export "rpu_alloc") (param i32 i32) (result i32) i32.const 0)
                (func (export "rpu_dealloc") (param i32 i32 i32))
                (func (export "rpu_run") (result i32) i32.const 0)
            )
        "#;

        let error = run_cli(wasm, &CapabilityConfig::default(), &[])
            .unwrap_err()
            .to_string();
        assert!(error.contains("unsupported WASM system import `rpu_system.unknown`"));
    }

    #[test]
    fn host_exit_stops_the_cartridge() {
        let wasm = br#"
            (module
                (import "rpu_system" "exit" (func $exit (param i32)))
                (memory (export "memory") 1)
                (func (export "rpu_abi_version") (result i32) i32.const 1)
                (func (export "rpu_alloc") (param i32 i32) (result i32) i32.const 0)
                (func (export "rpu_dealloc") (param i32 i32 i32))
                (func (export "rpu_run") (result i32)
                    i32.const 7
                    call $exit
                    i32.const 99)
            )
        "#;
        let requires = CapabilityConfig {
            system: true,
            graphics: false,
            audio: false,
            network: false,
        };

        assert_eq!(run_cli(wasm, &requires, &[]).unwrap(), 7);
    }

    #[test]
    fn initializes_a_wasm_module() {
        let wasm = br#"
            (module
                (memory (export "memory") 1)
                (func (export "rpu_abi_version") (result i32) i32.const 1)
                (func (export "rpu_alloc") (param i32 i32) (result i32) i32.const 0)
                (func (export "rpu_dealloc") (param i32 i32 i32))
                (func (export "rpu_module_init") (result i32) i32.const 0)
            )
        "#;

        load_module(wasm, &CapabilityConfig::default(), &[]).unwrap();
    }

    #[test]
    fn rejects_a_module_with_a_nonzero_init_status() {
        let wasm = br#"
            (module
                (memory (export "memory") 1)
                (func (export "rpu_abi_version") (result i32) i32.const 1)
                (func (export "rpu_alloc") (param i32 i32) (result i32) i32.const 0)
                (func (export "rpu_dealloc") (param i32 i32 i32))
                (func (export "rpu_module_init") (result i32) i32.const 4)
            )
        "#;

        let error = load_module(wasm, &CapabilityConfig::default(), &[])
            .err()
            .expect("module initialization should fail")
            .to_string();
        assert!(error.contains("status 4"));
    }

    #[test]
    fn rejects_a_module_with_an_unsupported_abi() {
        let wasm = br#"
            (module
                (memory (export "memory") 1)
                (func (export "rpu_abi_version") (result i32) i32.const 99)
                (func (export "rpu_alloc") (param i32 i32) (result i32) i32.const 0)
                (func (export "rpu_dealloc") (param i32 i32 i32))
                (func (export "rpu_module_init") (result i32) i32.const 0)
            )
        "#;

        let error = load_module(wasm, &CapabilityConfig::default(), &[])
            .err()
            .expect("ABI validation should fail")
            .to_string();
        assert!(error.contains("ABI version 99"));
    }
}
