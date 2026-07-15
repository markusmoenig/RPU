use rpu_core::wasm_abi;
use std::time::Instant;
use wasmer::{Function, FunctionEnv, FunctionEnvMut, Imports, Memory, RuntimeError, Store};

pub struct SystemService {
    env: FunctionEnv<SystemState>,
}

struct SystemState {
    args: Vec<Vec<u8>>,
    memory: Option<Memory>,
    started: Instant,
}

#[derive(Debug)]
struct ExitTrap(i32);

impl std::fmt::Display for ExitTrap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "cartridge exited with code {}", self.0)
    }
}

impl std::error::Error for ExitTrap {}

impl SystemService {
    pub fn new(store: &mut Store, args: &[String]) -> Self {
        Self {
            env: FunctionEnv::new(
                store,
                SystemState {
                    args: args
                        .iter()
                        .map(|argument| argument.as_bytes().to_vec())
                        .collect(),
                    memory: None,
                    started: Instant::now(),
                },
            ),
        }
    }

    pub fn namespace() -> &'static str {
        wasm_abi::SYSTEM_IMPORT_MODULE
    }

    pub fn supports_import(name: &str) -> bool {
        wasm_abi::system_imports()
            .iter()
            .any(|spec| spec.name == name)
    }

    pub fn register(&self, store: &mut Store, imports: &mut Imports) {
        let namespace = Self::namespace();
        imports.define(
            namespace,
            wasm_abi::IMPORT_ARG_COUNT,
            Function::new_typed_with_env(store, &self.env, host_arg_count),
        );
        imports.define(
            namespace,
            wasm_abi::IMPORT_ARG_LEN,
            Function::new_typed_with_env(store, &self.env, host_arg_len),
        );
        imports.define(
            namespace,
            wasm_abi::IMPORT_ARG_READ,
            Function::new_typed_with_env(store, &self.env, host_arg_read),
        );
        imports.define(
            namespace,
            wasm_abi::IMPORT_PRINT,
            Function::new_typed_with_env(store, &self.env, host_print),
        );
        imports.define(
            namespace,
            wasm_abi::IMPORT_EPRINT,
            Function::new_typed_with_env(store, &self.env, host_eprint),
        );
        imports.define(
            namespace,
            wasm_abi::IMPORT_EXIT,
            Function::new_typed_with_env(store, &self.env, host_exit),
        );
        imports.define(
            namespace,
            wasm_abi::IMPORT_NOW_MS,
            Function::new_typed_with_env(store, &self.env, host_now_ms),
        );
    }

    pub fn attach_memory(&self, store: &mut Store, memory: Memory) {
        self.env.as_mut(store).memory = Some(memory);
    }

    pub fn exit_code(error: &RuntimeError) -> Option<i32> {
        error.downcast_ref::<ExitTrap>().map(|exit| exit.0)
    }
}

fn host_arg_count(env: FunctionEnvMut<SystemState>) -> i32 {
    i32::try_from(env.data().args.len()).unwrap_or(i32::MAX)
}

fn host_arg_len(env: FunctionEnvMut<SystemState>, index: i32) -> i32 {
    argument(&env, index)
        .and_then(|value| i32::try_from(value.len()).ok())
        .unwrap_or(0)
}

fn host_arg_read(
    env: FunctionEnvMut<SystemState>,
    index: i32,
    ptr: i32,
    len: i32,
) -> Result<i32, RuntimeError> {
    let Some(argument) = argument(&env, index) else {
        return Ok(0);
    };
    let amount = argument.len().min(checked_len(len)?);
    write_guest(&env, ptr, &argument[..amount])?;
    Ok(i32::try_from(amount).unwrap_or(i32::MAX))
}

fn host_print(env: FunctionEnvMut<SystemState>, ptr: i32, len: i32) -> Result<(), RuntimeError> {
    let message = read_guest(&env, ptr, len)?;
    println!("{}", String::from_utf8_lossy(&message));
    Ok(())
}

fn host_eprint(env: FunctionEnvMut<SystemState>, ptr: i32, len: i32) -> Result<(), RuntimeError> {
    let message = read_guest(&env, ptr, len)?;
    eprintln!("{}", String::from_utf8_lossy(&message));
    Ok(())
}

fn host_exit(_env: FunctionEnvMut<SystemState>, code: i32) -> Result<(), RuntimeError> {
    Err(RuntimeError::user(Box::new(ExitTrap(code))))
}

fn host_now_ms(env: FunctionEnvMut<SystemState>) -> i32 {
    env.data().started.elapsed().as_millis() as i32
}

fn argument<'a>(env: &'a FunctionEnvMut<'_, SystemState>, index: i32) -> Option<&'a [u8]> {
    let index = usize::try_from(index).ok()?;
    env.data().args.get(index).map(Vec::as_slice)
}

fn read_guest(
    env: &FunctionEnvMut<SystemState>,
    ptr: i32,
    len: i32,
) -> Result<Vec<u8>, RuntimeError> {
    let memory = guest_memory(env)?;
    let view = memory.view(env);
    let ptr = checked_ptr(ptr)?;
    let len = checked_len(len)?;
    const MAX_HOST_STRING_BYTES: usize = 16 * 1024 * 1024;
    if len > MAX_HOST_STRING_BYTES {
        return Err(RuntimeError::new("guest string exceeds host limit"));
    }
    let end = ptr
        .checked_add(len as u64)
        .ok_or_else(|| RuntimeError::new("guest memory range overflow"))?;
    if end > view.data_size() {
        return Err(RuntimeError::new("guest memory range is out of bounds"));
    }
    let mut bytes = vec![0; len];
    view.read(ptr, &mut bytes)
        .map_err(|error| RuntimeError::new(format!("failed to read guest memory: {error}")))?;
    Ok(bytes)
}

fn write_guest(
    env: &FunctionEnvMut<SystemState>,
    ptr: i32,
    bytes: &[u8],
) -> Result<(), RuntimeError> {
    guest_memory(env)?
        .view(env)
        .write(checked_ptr(ptr)?, bytes)
        .map_err(|error| RuntimeError::new(format!("failed to write guest memory: {error}")))
}

fn guest_memory(env: &FunctionEnvMut<SystemState>) -> Result<Memory, RuntimeError> {
    env.data()
        .memory
        .clone()
        .ok_or_else(|| RuntimeError::new("guest memory is not initialized"))
}

fn checked_ptr(ptr: i32) -> Result<u64, RuntimeError> {
    u64::try_from(ptr).map_err(|_| RuntimeError::new("negative guest memory pointer"))
}

fn checked_len(len: i32) -> Result<usize, RuntimeError> {
    usize::try_from(len).map_err(|_| RuntimeError::new("negative guest memory length"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advertises_only_system_imports() {
        assert!(SystemService::supports_import(wasm_abi::IMPORT_PRINT));
        assert!(SystemService::supports_import(wasm_abi::IMPORT_NOW_MS));
        assert!(!SystemService::supports_import("graphics_create_buffer"));
    }
}
