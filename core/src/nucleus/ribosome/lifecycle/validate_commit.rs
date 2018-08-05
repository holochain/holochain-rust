use nucleus::ribosome::Runtime;
use wasmi::RuntimeArgs;
use wasmi::RuntimeValue;
use wasmi::Trap;
use nucleus::ribosome::runtime_allocate_encode_str;

pub fn invoke_validate_commit(
    runtime: &mut Runtime,
    _args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    runtime_allocate_encode_str(runtime, "")
}
