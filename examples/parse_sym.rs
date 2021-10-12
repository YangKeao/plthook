use std::ptr::null;

type PthreadCreateType = extern "C" fn(
    native: *mut libc::pthread_t,
    attr: *const libc::pthread_attr_t,
    f: extern "C" fn(*mut libc::c_void) -> *mut libc::c_void,
    value: *mut libc::c_void,
) -> libc::c_int;

static mut PTHREAD_CREATE_ADDR: *const libc::c_void = null();

extern "C" fn my_pthread_create(
    native: *mut libc::pthread_t,
    attr: *const libc::pthread_attr_t,
    f: extern "C" fn(*mut libc::c_void) -> *mut libc::c_void,
    value: *mut libc::c_void,
) -> libc::c_int {
    println!("before pthread_create");

    let pthread_create =
        unsafe { std::mem::transmute::<_, PthreadCreateType>(PTHREAD_CREATE_ADDR) };
    let ret = pthread_create(native, attr, f, value);

    println!("after pthread_create");

    ret
}

fn main() {
    std::thread::spawn(|| println!("not hooked"))
        .join()
        .unwrap();

    println!();

    let real_pthread_create_address = plthook::inject(
        my_pthread_create as *const libc::c_void,
        plthook::Target {
            function_name: "pthread_create".to_owned(),
            mmap: plthook::TargetMmap::Main,
        },
    );
    unsafe { PTHREAD_CREATE_ADDR = real_pthread_create_address };

    std::thread::spawn(|| println!("hooked")).join().unwrap();
}
