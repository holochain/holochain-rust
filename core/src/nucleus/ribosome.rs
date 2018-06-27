// In this example we execute a contract funciton exported as "_call"

extern crate wabt;
extern crate wasmi;

// use std::mem::transmute;
use std::sync::mpsc::{channel, Sender};
// use ::agent::Action::Commit;
use ::state::State;
use ::state::Action;
// use ::state::ActionWrapper;
use ::instance::Observer;
use ::instance::DISPATCH_WITHOUT_CHANNELS;

use state;

use self::wasmi::{
    Error as InterpreterError, Externals, FuncInstance, FuncRef, ImportsBuilder,
    ModuleImportResolver, ModuleInstance, RuntimeArgs, RuntimeValue, Signature, Trap, ValueType,
};


/// Object to hold VM data that we want out of the VM
#[derive(Clone)]
pub struct Runtime {
    print_output:     Vec<u32>,
    pub result:       String,
    action_channel:   Sender<state::ActionWrapper>,
    observer_channel: Sender<Observer>,
}


/// List of all the API functions available in Nucleus
#[repr(usize)]
enum HcApiFuncIndex {
    /// Print debug information in the console
    /// print()
    PRINT = 0,
    /// Commit an entry to source chain
    /// commit(entry_type : String, entry_content : String)
    COMMIT,
    // Add new API function index here
    // ...
}



/// Send Action to Instance's Event Queue and block until is has been processed.
pub fn dispatch_action_and_wait(action_channel:   &Sender<state::ActionWrapper>,
                            observer_channel: &Sender<Observer>,
                            action:           Action)
{
    // Wrap Action
    let wrapper = state::ActionWrapper::new(action);
    let wrapper_clone = wrapper.clone();

    // Create blocking channel
    let (sender, receiver) = channel::<bool>();

    // Create blocking observer
    let closure = move |state: &State| {
        if state.history.contains(&wrapper_clone) {
            sender
              .send(true)
              .unwrap_or_else(|_| panic!(DISPATCH_WITHOUT_CHANNELS));
            true
        } else {
            false
        }
    };
    let observer = Observer {
        sensor: Box::new(closure),
        done: false,
    };

    // Send observer to instance
    observer_channel
        .send(observer)
        .unwrap_or_else(|_| panic!(DISPATCH_WITHOUT_CHANNELS));

    // Send action to instance
    action_channel
        .send(wrapper)
        .unwrap_or_else(|_| panic!(DISPATCH_WITHOUT_CHANNELS));

    // Block until Observer has sensed the completion of the Action
    receiver
      .recv()
      .unwrap_or_else(|_| panic!(DISPATCH_WITHOUT_CHANNELS));
}



// WASM modules = made to be run browser along side Javascript modules
// import and export in strings
/// Executes an exposed function
#[allow(dead_code)]
pub fn call(action_channel:   &Sender<state::ActionWrapper>,
            observer_channel: &Sender<Observer>,
            wasm:             Vec<u8>,
            function_name:    &str)
  -> Result<Runtime, InterpreterError>
{
    let module = wasmi::Module::from_buffer(wasm).unwrap();

    impl Externals for Runtime {
        fn invoke_index(
            &mut self,
            index: usize,
            args: RuntimeArgs,
        )
            -> Result<Option<RuntimeValue>, Trap>
        {
            match index {
                index if index == HcApiFuncIndex::PRINT as usize => {
                    let arg: u32 = args.nth(0);
                    self.print_output.push(arg);
                    Ok(None)
                }
                index if index == HcApiFuncIndex::COMMIT as usize => {
                    // FIXME unpack args into Entry struct with serializer
                    let entry = ::common::entry::Entry::new("FIXME - content string here");

                    // Create commit Action
                    let action_commit = ::state::Action::Agent(::agent::Action::Commit(entry.clone()));

                    // Send Action and block for result
                    dispatch_action_and_wait(&self.action_channel, &self.observer_channel, action_commit.clone());

                    Ok(None) // FIXME - Change to Result<Runtime, InterpreterError>?
                }
                // Add API function code here
                // ....
                _ => panic!("unknown function index"),
            }
        }
    }

    struct RuntimeModuleImportResolver;

    impl ModuleImportResolver for RuntimeModuleImportResolver {
        fn resolve_func(
            &self,
            field_name: &str,
            _signature: &Signature,
        ) -> Result<FuncRef, InterpreterError>
        {
            let func_ref = match field_name {
                "print" => FuncInstance::alloc_host(
                    Signature::new(&[ValueType::I32][..], None),
                    HcApiFuncIndex::PRINT as usize,
                ),
                "commit" => FuncInstance::alloc_host(
                      Signature::new(&[ValueType::I32][..], None),
                      HcApiFuncIndex::COMMIT as usize,
                ),
                // Add API function here
                // ....
                _ => {
                    return Err(InterpreterError::Function(format!(
                        "host module doesn't export function with name {}",
                        field_name
                    )))
                }
            };
            Ok(func_ref)
        }
    }

    let mut imports = ImportsBuilder::new();
    imports.push_resolver("env", &RuntimeModuleImportResolver);

    let main = ModuleInstance::new(&module, &imports)
        .expect("Failed to instantiate module")
        .assert_no_start();

    let memory = main
        .export_by_name("memory")
        .expect("all modules compiled with rustc should have an export named 'memory'; qed")
        .as_memory()
        .expect("in module generated by rustc export named 'memory' should be a memory; qed")
        .clone();

    let parameters = vec![6u8, 7u8, 8u8];
    memory
        .set(0, &parameters)
        .expect("memory should be writable");

    let mut runtime = Runtime {
        print_output: vec![],
        result: String::new(),
        action_channel : action_channel.clone(),
        observer_channel : observer_channel.clone(),
    };
    let i32_result: i32 = main
        .invoke_export(function_name, &[], &mut runtime)?
        .unwrap()
        .try_into::<i32>()
        .unwrap();
    runtime.result = i32_result.to_string();
    Ok(runtime.clone())
}


//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use wabt::Wat2Wasm;

    fn _test_wasm_from_file() -> Vec<u8> {
        use std::io::prelude::*;
        let mut file = File::open(
            "src/nucleus/wasm-test/target/wasm32-unknown-unknown/release/wasm_ribosome_test.wasm",
        ).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        return buf;
    }

    fn test_wasm() -> Vec<u8> {
        let wasm_binary = Wat2Wasm::new()
            .canonicalize_lebs(false)
            .write_debug_names(true)
            .convert(
                r#"
                (module
                    (type (;0;) (func (result i32)))
                    (type (;1;) (func (param i32)))
                    (type (;2;) (func))
                    (import "env" "print" (func $print (type 1)))
                    (func $test_print (type 0) (result i32)
                        i32.const 1337
                        call $print
                        i32.const 0)
                    (func $rust_eh_personality (type 2))
                    (table (;0;) 1 1 anyfunc)
                    (memory (;0;) 17)
                    (global (;0;) (mut i32) (i32.const 1049600))
                    (export "memory" (memory 0))
                    (export "test_print" (func $test_print))
                    (export "rust_eh_personality" (func $rust_eh_personality)))
            "#,
            )
            .unwrap();

        wasm_binary.as_ref().to_vec()
    }

    #[test]
    fn test_print() {
        let (action_channel, _ ) = channel::<::state::ActionWrapper>();
        let (tx_observer, rx_observer) = channel::<Observer>();
        let runtime = call(&action_channel, &tx_observer, test_wasm(), "test_print").expect("test_print should be callable");
        assert_eq!(runtime.print_output.len(), 1);
        assert_eq!(runtime.print_output[0], 1337)
    }
}
