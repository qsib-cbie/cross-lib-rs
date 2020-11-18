#![no_std]
#![feature(default_alloc_error_handler)]
#![feature(lang_items)]

extern crate alloc;
use alloc::vec::Vec;
use cstr_core::{c_char, CString, CStr};

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
#[allow(unused_imports)]
use std::prelude::*;

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(target_arch = "arm")]
use alloc_cortex_m::CortexMHeap;

#[cfg(target_arch = "arm")]
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[cfg(target_arch = "arm")]
#[no_mangle]
pub extern fn rust_init() {
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024; // in bytes
    unsafe { ALLOCATOR.init(start, size) }
}

#[cfg(any(target_arch = "aarch64", target_arch="x86_64"))]
use jemallocator::Jemalloc;

#[cfg(any(target_arch = "aarch64", target_arch="x86_64"))]
#[global_allocator]
static ALLOCATOR: Jemalloc = Jemalloc;

#[cfg(not(test))]
#[cfg(any(target_arch = "aarch64", target_arch="x86_64"))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() { }

#[cfg(any(target_arch = "aarch64", target_arch="x86_64"))]
#[no_mangle]
extern "C" fn rust_oom() { }


#[cfg(any(target_arch = "aarch64", target_arch="x86_64"))]
#[no_mangle]
pub extern fn rust_init() {
    // nop, use default jemalloc
}

#[no_mangle]
pub extern fn rust_greeting(to: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(to) };
    let recipient = match c_str.to_str() {
        Err(_) => "there",
        Ok(string) => string,
    };

    let response_message = "Hello ";
    let mut response = Vec::<u8>::with_capacity(response_message.len() + recipient.len());
    response_message.as_bytes().into_iter().for_each(|c| response.push(*c));
    recipient.as_bytes().into_iter().for_each(|c| response.push(*c));

    CString::new(response)
        .unwrap()
        .into_raw()
}

#[no_mangle]
pub extern fn rust_greeting_free(s: *mut c_char) {
    unsafe {
        if s.is_null() { return }
        CString::from_raw(s)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn say_hello() {
        rust_init();
        let name = CString::new("World").unwrap();
        let s = rust_greeting(name.as_ptr());
        unsafe {
            let greeting = CStr::from_ptr(s as *const c_char);
            println!("Rust Test says {:#?}", greeting);
        }
        rust_greeting_free(s);
    }
}