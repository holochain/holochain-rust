extern crate newrelic;



#[new_relic_proc_macro::trace("SIM2H_SERVER","TRANSACTION_tYPE","CATEGORY")]
pub fn wrapped_test_function(_caption:Option<u8>) 
{
    println!("stuff")
}

#[test]
fn test_macro() {
    wrapped_test_function(None);
}