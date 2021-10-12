use std::convert::TryInto;
use std::env;
use std::fs::File;
use std::os::unix::prelude::AsRawFd;
use std::path::PathBuf;
use std::ptr;
use std::ptr::null;
use std::slice;

use libc::c_void;
use object::Object;
use object::ObjectSymbol;
use object::ObjectSymbolTable;

extern "C" {
    fn base_addr() -> u64;
}

pub enum TargetMmap {
    Main,
    SharedLibrary(String),
}

pub struct Target {
    pub function_name: String,
    pub mmap: TargetMmap,
}

pub fn inject(hook: *const libc::c_void, target: Target) -> *const libc::c_void {
    let path: PathBuf = match target.mmap {
        TargetMmap::Main => env::current_exe().map(|e| e.into()).unwrap_or_default(),

        TargetMmap::SharedLibrary(_) => {
            unimplemented!()
        }
    };

    let file = File::open(&path).unwrap();
    return inject_file(target.function_name, hook, file);
}

fn inject_file(
    function_name: String,
    hook: *const libc::c_void,
    file: File,
) -> *const libc::c_void {
    let len: usize = file.metadata().unwrap().len().try_into().unwrap();
    let ptr = unsafe {
        libc::mmap(
            ptr::null_mut(),
            len,
            libc::PROT_READ,
            libc::MAP_PRIVATE,
            file.as_raw_fd(),
            0,
        )
    };
    if ptr == libc::MAP_FAILED {
        // TODO: handle this error
        unimplemented!()
    }

    let data = unsafe { slice::from_raw_parts(ptr as *const u8, len) };
    let elf = object::File::parse(data).unwrap();

    for (offset, rel) in elf.dynamic_relocations().unwrap() {
        match rel.target() {
            object::RelocationTarget::Symbol(sym) => {
                let symbol = elf
                    .dynamic_symbol_table()
                    .unwrap()
                    .symbol_by_index(sym)
                    .unwrap();

                if symbol.name().unwrap().contains(&function_name) {
                    let addr = offset + unsafe { base_addr() };

                    unsafe {
                        libc::mprotect(
                            (addr / 4096 * 4096) as *mut c_void,
                            4096,
                            libc::PROT_READ | libc::PROT_WRITE,
                        );
                    }

                    let pthread_create_ptr = addr as *mut *const libc::c_void;
                    let ret = unsafe { *pthread_create_ptr };
                    unsafe {
                        *pthread_create_ptr = hook;
                    }

                    return ret;
                }
            }
            _ => {}
        }
    }

    return null();
}
