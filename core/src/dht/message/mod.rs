pub mod response_error;

pub trait Message {

    fn type_name(&self) -> &str;

    fn type_code(&self) -> i8;

    fn time(&self) -> String;

    fn from(&self) -> String;

    fn body(&self) -> String;

}
