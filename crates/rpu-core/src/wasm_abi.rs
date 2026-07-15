use crate::{CapabilityConfig, ProjectKind};

pub const ABI_VERSION: u32 = 1;
pub const MEMORY_EXPORT: &str = "memory";
pub const SYSTEM_IMPORT_MODULE: &str = "rpu_system";
pub const GRAPHICS_IMPORT_MODULE: &str = "rpu_graphics";

pub const EXPORT_ABI_VERSION: &str = "rpu_abi_version";
pub const EXPORT_ALLOC: &str = "rpu_alloc";
pub const EXPORT_DEALLOC: &str = "rpu_dealloc";
pub const EXPORT_RUN: &str = "rpu_run";
pub const EXPORT_START: &str = "rpu_start";
pub const EXPORT_UPDATE: &str = "rpu_update";
pub const EXPORT_STOP: &str = "rpu_stop";
pub const EXPORT_MODULE_INIT: &str = "rpu_module_init";

pub const IMPORT_ARG_COUNT: &str = "arg_count";
pub const IMPORT_ARG_LEN: &str = "arg_len";
pub const IMPORT_ARG_READ: &str = "arg_read";
pub const IMPORT_PRINT: &str = "print";
pub const IMPORT_EPRINT: &str = "eprint";
pub const IMPORT_EXIT: &str = "exit";
pub const IMPORT_NOW_MS: &str = "now_ms";

pub const IMPORT_GRAPHICS_BEGIN_FRAME: &str = "begin_frame";
pub const IMPORT_GRAPHICS_CLEAR: &str = "clear";
pub const IMPORT_GRAPHICS_DRAW_RECT: &str = "draw_rect";
pub const IMPORT_GRAPHICS_END_FRAME: &str = "end_frame";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmValueType {
    I32,
    F32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmFunctionKind {
    Export,
    Import,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WasmFunctionSpec {
    pub kind: WasmFunctionKind,
    pub module: Option<&'static str>,
    pub name: &'static str,
    pub params: &'static [WasmValueType],
    pub results: &'static [WasmValueType],
}

impl WasmFunctionSpec {
    pub const fn export(
        name: &'static str,
        params: &'static [WasmValueType],
        results: &'static [WasmValueType],
    ) -> Self {
        Self {
            kind: WasmFunctionKind::Export,
            module: None,
            name,
            params,
            results,
        }
    }

    pub const fn import(
        module: &'static str,
        name: &'static str,
        params: &'static [WasmValueType],
        results: &'static [WasmValueType],
    ) -> Self {
        Self {
            kind: WasmFunctionKind::Import,
            module: Some(module),
            name,
            params,
            results,
        }
    }
}

pub fn common_required_exports() -> Vec<WasmFunctionSpec> {
    vec![
        WasmFunctionSpec::export(EXPORT_ABI_VERSION, &[], &[WasmValueType::I32]),
        WasmFunctionSpec::export(
            EXPORT_ALLOC,
            &[WasmValueType::I32, WasmValueType::I32],
            &[WasmValueType::I32],
        ),
        WasmFunctionSpec::export(
            EXPORT_DEALLOC,
            &[WasmValueType::I32, WasmValueType::I32, WasmValueType::I32],
            &[],
        ),
    ]
}

pub fn required_exports_for_kind(kind: ProjectKind) -> Vec<WasmFunctionSpec> {
    let mut exports = common_required_exports();
    match kind {
        ProjectKind::Cli => {
            exports.push(WasmFunctionSpec::export(
                EXPORT_RUN,
                &[],
                &[WasmValueType::I32],
            ));
        }
        ProjectKind::App => {
            exports.push(WasmFunctionSpec::export(
                EXPORT_START,
                &[],
                &[WasmValueType::I32],
            ));
            exports.push(WasmFunctionSpec::export(
                EXPORT_UPDATE,
                &[WasmValueType::F32],
                &[WasmValueType::I32],
            ));
            exports.push(WasmFunctionSpec::export(EXPORT_STOP, &[], &[]));
        }
        ProjectKind::Module => {
            exports.push(WasmFunctionSpec::export(
                EXPORT_MODULE_INIT,
                &[],
                &[WasmValueType::I32],
            ));
        }
    }
    exports
}

pub fn required_imports_for_capabilities(requires: &CapabilityConfig) -> Vec<WasmFunctionSpec> {
    let mut imports = Vec::new();
    if requires.system {
        imports.extend(system_imports());
    }
    if requires.graphics {
        imports.extend(graphics_imports());
    }
    imports
}

pub fn system_imports() -> Vec<WasmFunctionSpec> {
    vec![
        WasmFunctionSpec::import(
            SYSTEM_IMPORT_MODULE,
            IMPORT_ARG_COUNT,
            &[],
            &[WasmValueType::I32],
        ),
        WasmFunctionSpec::import(
            SYSTEM_IMPORT_MODULE,
            IMPORT_ARG_LEN,
            &[WasmValueType::I32],
            &[WasmValueType::I32],
        ),
        WasmFunctionSpec::import(
            SYSTEM_IMPORT_MODULE,
            IMPORT_ARG_READ,
            &[WasmValueType::I32, WasmValueType::I32, WasmValueType::I32],
            &[WasmValueType::I32],
        ),
        WasmFunctionSpec::import(
            SYSTEM_IMPORT_MODULE,
            IMPORT_PRINT,
            &[WasmValueType::I32, WasmValueType::I32],
            &[],
        ),
        WasmFunctionSpec::import(
            SYSTEM_IMPORT_MODULE,
            IMPORT_EPRINT,
            &[WasmValueType::I32, WasmValueType::I32],
            &[],
        ),
        WasmFunctionSpec::import(
            SYSTEM_IMPORT_MODULE,
            IMPORT_EXIT,
            &[WasmValueType::I32],
            &[],
        ),
        WasmFunctionSpec::import(
            SYSTEM_IMPORT_MODULE,
            IMPORT_NOW_MS,
            &[],
            &[WasmValueType::I32],
        ),
    ]
}

pub fn graphics_imports() -> Vec<WasmFunctionSpec> {
    use WasmValueType::{F32, I32};

    vec![
        WasmFunctionSpec::import(
            GRAPHICS_IMPORT_MODULE,
            IMPORT_GRAPHICS_BEGIN_FRAME,
            &[I32, I32],
            &[],
        ),
        WasmFunctionSpec::import(
            GRAPHICS_IMPORT_MODULE,
            IMPORT_GRAPHICS_CLEAR,
            &[F32, F32, F32, F32],
            &[],
        ),
        WasmFunctionSpec::import(
            GRAPHICS_IMPORT_MODULE,
            IMPORT_GRAPHICS_DRAW_RECT,
            &[F32, F32, F32, F32, F32, F32, F32, F32],
            &[],
        ),
        WasmFunctionSpec::import(GRAPHICS_IMPORT_MODULE, IMPORT_GRAPHICS_END_FRAME, &[], &[]),
    ]
}
