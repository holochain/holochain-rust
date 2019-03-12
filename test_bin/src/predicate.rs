/// Macro for transforming a type check into a predicate
macro_rules! one_is {
    ($p:pat) => {
        |d| {
            if let $p = d {
                return true;
            }
            return false;
        }
    };
}

macro_rules! one_is_where {
    ($p:pat, $code:tt) => {
        |d| {
            return if let $p = d {
                $code
            } else {
                false
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! one_let {
    ($p:pat = $enum:ident $code:tt) => {
        if let $p = $enum {
            $code
        } else {
            unimplemented!();
        }
    };
}
