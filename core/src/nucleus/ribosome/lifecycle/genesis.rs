use nucleus::ribosome::Runtime;
use wasmi::RuntimeArgs;
use wasmi::Trap;
use wasmi::RuntimeValue;
use nucleus::ribosome::runtime_allocate_encode_str;

pub fn invoke_genesis(
    runtime: &mut Runtime,
    _args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    runtime_allocate_encode_str(runtime, "")
}
