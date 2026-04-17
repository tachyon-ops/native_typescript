use libffi::middle::{Arg, Cif, Type};
use std::ffi::CString;
use crate::loader::FfiError;

#[derive(Debug, Clone)]
pub enum FfiValue<'a> {
    Number(f64),
    Bool(bool),
    String(&'a str),
    Null,
    Undefined,
}

pub fn invoke_native<'a>(func_ptr: *mut std::ffi::c_void, args: &[FfiValue<'a>]) -> Result<FfiValue<'a>, FfiError> {
    // In a full implementation, we need to map the return type dynamically.
    // For this prototype, we'll assume f64 (Number) return just to prove the bridge.

    // Storage for temporaries that must outlive the `cif.call` invocation.
    let mut f64_storage = Vec::new();
    let mut string_storage = Vec::new();
    let mut bool_storage = Vec::new();
    let mut ptr_storage = Vec::new();

    for a in args {
        match a {
            FfiValue::Number(n) => f64_storage.push(*n),
            FfiValue::String(s) => {
                let c_str = CString::new(*s).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
                string_storage.push(c_str);
            }
            FfiValue::Bool(b) => bool_storage.push(if *b { 1u8 } else { 0u8 }),
            FfiValue::Null | FfiValue::Undefined => {
                let ptr: *const std::ffi::c_void = std::ptr::null();
                ptr_storage.push(ptr);
            }
        }
    }

    let mut ffi_args = Vec::new();
    let mut ffi_types = Vec::new();

    let mut c_f64 = 0;
    let mut c_str = 0;
    let mut c_bool = 0;
    let mut c_ptr = 0;

    for a in args {
        match a {
            FfiValue::Number(_) => {
                ffi_args.push(Arg::new(&f64_storage[c_f64]));
                ffi_types.push(Type::f64());
                c_f64 += 1;
            }
            FfiValue::String(_) => {
                let ptr = string_storage[c_str].as_ptr();
                ptr_storage.push(ptr as *const std::ffi::c_void);
                let last_idx = ptr_storage.len() - 1;
                ffi_args.push(Arg::new(&ptr_storage[last_idx]));
                ffi_types.push(Type::pointer());
                c_str += 1;
            }
            FfiValue::Null | FfiValue::Undefined => {
                let last_idx = ptr_storage.len() - c_ptr - 1; 
                ffi_args.push(Arg::new(&ptr_storage[last_idx]));
                ffi_types.push(Type::pointer());
                c_ptr += 1;
            }
            FfiValue::Bool(_) => {
                ffi_args.push(Arg::new(&bool_storage[c_bool]));
                ffi_types.push(Type::u8());
                c_bool += 1;
            }
        }
    }

    // Call the function via CIF
    let cif = Cif::new(ffi_types.into_iter(), Type::f64());
    
    let result: f64 = unsafe {
        cif.call(libffi::middle::CodePtr::from_ptr(func_ptr), &ffi_args)
    };

    Ok(FfiValue::Number(result))
}
