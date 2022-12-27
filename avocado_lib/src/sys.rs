use crate::incl::*;

pub fn run_once(mut ran: Local<bool>) -> bool {
    if !*ran {
        *ran = true;
        true
    } else {
        false
    }
}
