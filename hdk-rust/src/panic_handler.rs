use crate::api::debug;
use holochain_core_types::json::RawString;
use std::panic;

#[no_mangle]
pub extern "C" fn __install_panic_handler() -> () {
    panic::set_hook(Box::new(move |info| {
        let _ = debug(RawString::from(
            info.payload().downcast_ref::<String>().unwrap().clone(),
        ));
        //let _ = debug(RawString::from(format!("{}", info.message().unwrap().clone())));
        let _ = if let Some(location) = info.location() {
            debug(RawString::from(format!(
                "panic occurred in file '{}' at line {}",
                location.file(),
                location.line()
            )))
        } else {
            debug(RawString::from(format!(
                "panic occurred but can't get location information..."
            )))
        };
    }));
}
