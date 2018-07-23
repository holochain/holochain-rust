/// HcApiFuncIndex::PRINT function code
pub fn invoke_print(runtime: &mut Runtime, args: &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {
    let arg: u32 = args.nth(0);
    runtime.print_output.push(arg);
    Ok(None)
}
