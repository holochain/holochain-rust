use action::Action;
use nucleus::FunctionCall;

pub struct ExecuteZomeFunction {
    call: FunctionCall,
}

impl Action for ExecuteZomeFunction {}

impl ExecuteZomeFunction {

    pub fn new(call: &FunctionCall) {
        ExecuteZomeFunction{
            call: call.clone(),
        }
    }

}
