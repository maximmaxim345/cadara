//! Remaining libc and libc++ implementation of `wasm32-unknown-unknown-libcxx`
//!
//! This library provides implementations of functions required for `libc` and `libc++`
//! to work.
//!
//! Calling `wasm_libc::init()` before using calling a C++ library is required (in most cases).
#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(clippy::module_inception)]
#![allow(clippy::needless_lifetimes)]
#![allow(unused_variables)]

static INIT: std::sync::Once = std::sync::Once::new();

/// Call C++ constructors
///
/// This function must be called on `wasm32-unknown-unknown` before using any C++ code containing global objects.
///
/// On any platform other than `wasm32-unknown-unknown` this function does nothing.
/// It is safe to call multiple times.
pub fn init() {
    // Compare-and-swap operation ensures thread-safety and single initialization
    #[cfg(target_arch = "wasm32")]
    INIT.call_once(|| unsafe {
        extern "C" {
            fn __wasm_call_ctors();
        }
        __wasm_call_ctors();
    });
}

// TODO(wasm32): implement unimplemented functions
// TODO(wasm32): test if some functions can be removed/implemented in C/C++
// TODO(wasm32): move this crate out of this repo into a separate library
// TODO(wasm32): exceptions are broken, fix them
// TODO(wasm32): verify that these functions are safe
#[cfg(target_arch = "wasm32")]
mod api {
    use log::{debug, error, info, trace, warn};
    use std::ffi::CStr;
    use std::process;

    const ALIGN: usize = 8;

    #[no_mangle]
    pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
        // Check for overflow when adding ALIGN
        let total_size = match size.checked_add(ALIGN) {
            Some(size) => size,
            None => return std::ptr::null_mut(),
        };

        let layout = match std::alloc::Layout::from_size_align(total_size, ALIGN) {
            Ok(layout) => layout,
            Err(_) => return std::ptr::null_mut(),
        };

        let ptr = std::alloc::alloc(layout);
        if ptr.is_null() {
            return std::ptr::null_mut();
        }

        *(ptr as *mut usize) = size;
        // Safe offset calculation
        if (ptr as usize).checked_add(ALIGN).is_none() {
            std::alloc::dealloc(ptr, layout);
            return std::ptr::null_mut();
        }
        ptr.add(ALIGN)
    }

    #[no_mangle]
    pub unsafe extern "C" fn calloc(nmemb: i32, size: i32) -> i32 {
        if nmemb < 0 || size < 0 {
            return 0;
        }

        // Check for overflow in multiplication
        let total_size = match (nmemb as usize).checked_mul(size as usize) {
            Some(size) => size,
            None => return 0,
        };

        // Handle zero size
        if total_size == 0 {
            return 0;
        }

        // Allocate memory using malloc
        let ptr = malloc(total_size);
        if ptr.is_null() {
            return 0;
        }

        // Zero out the allocated memory
        std::ptr::write_bytes(ptr, 0, total_size);

        // Return the pointer as i32
        ptr as i32
    }

    #[no_mangle]
    pub unsafe extern "C" fn free(ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }

        // Calculate base pointer
        let base_ptr = ptr.sub(ALIGN);
        let size = *(base_ptr as *mut usize);

        // Check for overflow when adding ALIGN to size
        let total_size = match size.checked_add(ALIGN) {
            Some(s) => s,
            None => return,
        };

        let layout = match std::alloc::Layout::from_size_align(total_size, ALIGN) {
            Ok(layout) => layout,
            Err(_) => return,
        };

        std::alloc::dealloc(base_ptr, layout);
    }

    #[no_mangle]
    pub unsafe extern "C" fn realloc(ptr: *mut u8, new_size: usize) -> *mut u8 {
        if ptr.is_null() {
            return malloc(new_size);
        }

        let base_ptr = ptr.sub(ALIGN);
        let old_size = *(base_ptr as *mut usize);

        // Check for overflow when adding ALIGN
        let old_total_size = match old_size.checked_add(ALIGN) {
            Some(s) => s,
            None => return std::ptr::null_mut(),
        };

        let new_total_size = match new_size.checked_add(ALIGN) {
            Some(s) => s,
            None => return std::ptr::null_mut(),
        };

        let layout = match std::alloc::Layout::from_size_align(old_total_size, ALIGN) {
            Ok(layout) => layout,
            Err(_) => return std::ptr::null_mut(),
        };

        let new_ptr = std::alloc::realloc(base_ptr, layout, new_total_size);
        if new_ptr.is_null() {
            return std::ptr::null_mut();
        }

        *(new_ptr as *mut usize) = new_size;

        // Safe offset calculation
        if (new_ptr as usize).checked_add(ALIGN).is_none() {
            let layout = std::alloc::Layout::from_size_align_unchecked(new_total_size, ALIGN);
            std::alloc::dealloc(new_ptr, layout);
            return std::ptr::null_mut();
        }
        new_ptr.add(ALIGN)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __cxa_allocate_exception(thrown_size: usize) -> *mut u8 {
        // Allocate memory for the exception object
        // This is a stub - exceptions won't actually work, but we need to return
        // valid memory to avoid immediate crashes. The exception will be "thrown"
        // in __cxa_throw which will abort.
        malloc(thrown_size)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __cxa_throw(
        _thrown_exception: *mut u8,
        _tinfo: *mut u8,
        _dest: *mut u8,
    ) -> ! {
        // Exception handling is not yet implemented for WASM.
        // According to OpenCASCADE documentation, exceptions should not be thrown
        // during normal execution, but they're still present in the code.
        error!(
            "Exception thrown in WASM - this should not happen during normal OpenCASCADE execution"
        );
        error!("If you see this, there may be an error in the CAD operations");
        process::abort();
    }

    #[no_mangle]
    pub unsafe extern "C" fn __cxa_atexit(a: i32, b: i32, c: i32) -> i32 {
        10
    }

    #[no_mangle]
    pub unsafe extern "C" fn tolower(c: i32) -> i32 {
        if c >= 'A' as i32 && c <= 'Z' as i32 {
            c + 32
        } else {
            c
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn longjmp(_buf: i32, _value: i32) {
        error!("longjmp not implemented");
        panic!("longjmp not implemented");
    }

    #[no_mangle]
    pub unsafe extern "C" fn pthread_mutexattr_init(attr: i32) -> i32 {
        0 // Success
    }

    #[no_mangle]
    pub unsafe extern "C" fn pthread_mutexattr_settype(attr: i32, type_: i32) -> i32 {
        0 // Success
    }

    #[no_mangle]
    pub unsafe extern "C" fn pthread_mutexattr_destroy(attr: i32) -> i32 {
        0 // Success
    }

    #[no_mangle]
    pub unsafe extern "C" fn __flt_rounds() -> i32 {
        1 // FE_TONEAREST
    }

    #[no_mangle]
    pub unsafe extern "C" fn isspace(c: i32) -> i32 {
        if (c as u8).is_ascii_whitespace() {
            1
        } else {
            0
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn lround(x: f64) -> i32 {
        x.round() as i32
    }

    #[no_mangle]
    pub unsafe extern "C" fn gettimeofday(_tv: i32, _tz: i32) -> i32 {
        error!("gettimeofday not implemented");
        -1
    }

    #[no_mangle]
    pub unsafe extern "C" fn times(_buf: i32) -> i64 {
        error!("times not implemented");
        -1
    }

    #[no_mangle]
    pub unsafe extern "C" fn wmemmove(dest: i32, src: i32, n: i32) -> i32 {
        // Basic implementation - would need proper wide char handling
        std::ptr::copy(src as *const u8, dest as *mut u8, (n * 4) as usize);
        dest
    }

    #[no_mangle]
    pub unsafe extern "C" fn wcslen(s: i32) -> i32 {
        let mut len = 0;
        let mut ptr = s as *const i32;
        while *ptr != 0 {
            len += 1;
            ptr = ptr.add(1);
        }
        len
    }

    // For the remaining complex functions, we'll log errors and panic
    macro_rules! unimplemented_function {
            ($name:ident, $($arg:ident: $type:ty),*) => {
                #[no_mangle]
                pub unsafe extern "C" fn $name($($arg: $type),*) -> i32 {
                    error!(concat!(stringify!($name), " not implemented"));
                    panic!(concat!(stringify!($name), " not implemented"));
                }
            }
        }

    unimplemented_function!(vsscanf, s: i32, format: i32, ap: i32);
    unimplemented_function!(vsnprintf, s: i32, n: i32, format: i32, ap: i32);
    unimplemented_function!(sscanf, s: i32, format: i32, args: i32);
    unimplemented_function!(vasprintf, strp: i32, fmt: i32, ap: i32);

    #[no_mangle]
    pub unsafe extern "C" fn wmemcpy(dest: i32, src: i32, n: i32) -> i32 {
        std::ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, (n * 4) as usize);
        dest
    }

    #[no_mangle]
    pub unsafe extern "C" fn wmemset(dest: i32, c: i32, n: i32) -> i32 {
        let ptr = dest as *mut i32;
        for i in 0..n {
            *ptr.add(i as usize) = c;
        }
        dest
    }

    #[no_mangle]
    pub unsafe extern "C" fn strtoll(_nptr: i32, _endptr: i32, _base: i32) -> i64 {
        error!("strtoll not implemented");
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn strtoull(_nptr: i32, _endptr: i32, _base: i32) -> i64 {
        error!("strtoull not implemented");
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn strtof(_nptr: i32, _endptr: i32) -> f32 {
        error!("strtof not implemented");
        0.0
    }

    #[no_mangle]
    pub unsafe extern "C" fn strtod(_nptr: i32, _endptr: i32) -> f64 {
        error!("strtod not implemented");
        0.0
    }

    #[no_mangle]
    pub unsafe extern "C" fn strtold(_nptr: i32, _endptr: i32, _unused: i32) {
        error!("strtold not implemented");
        panic!("strtold not implemented");
    }

    #[no_mangle]
    pub unsafe extern "C" fn snprintf(_s: i32, _n: i32, _format: i32, _args: i32) -> i32 {
        error!("snprintf not implemented");
        -1
    }

    #[no_mangle]
    pub unsafe extern "C" fn isascii(c: i32) -> i32 {
        if c >= 0 && c <= 127 {
            1
        } else {
            0
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn ungetc(_c: i32, _stream: i32) -> i32 {
        error!("ungetc not implemented");
        -1
    }

    #[no_mangle]
    pub unsafe extern "C" fn getc(_stream: i32) -> i32 {
        error!("getc not implemented");
        -1
    }

    #[no_mangle]
    pub unsafe extern "C" fn isxdigit_l(c: i32, _locale: i32) -> i32 {
        if (c as u8).is_ascii_hexdigit() {
            1
        } else {
            0
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn isdigit_l(c: i32, _locale: i32) -> i32 {
        if (c as u8).is_ascii_digit() {
            1
        } else {
            0
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn strftime_l(
        _s: i32,
        _maxsize: i32,
        _format: i32,
        _timeptr: i32,
        _locale: i32,
    ) -> i32 {
        error!("strftime_l not implemented");
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn iswlower_l(c: i32, _locale: i32) -> i32 {
        if (c as u8).is_ascii_lowercase() {
            1
        } else {
            0
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn islower_l(c: i32, _locale: i32) -> i32 {
        if (c as u8).is_ascii_lowercase() {
            1
        } else {
            0
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn isupper_l(c: i32, _locale: i32) -> i32 {
        if (c as u8).is_ascii_uppercase() {
            1
        } else {
            0
        }
    }

    // Complex multibyte/wide character conversion functions
    macro_rules! unimplemented_mb_function {
            ($name:ident, $($arg:ident: $type:ty),*) => {
                #[no_mangle]
                pub unsafe extern "C" fn $name($($arg: $type),*) -> i32 {
                    error!(concat!(stringify!($name), " not implemented"));
                    -1
                }
            }
        }

    unimplemented_mb_function!(wcsnrtombs, dest: i32, src: i32, nwc: i32, len: i32, ps: i32);
    unimplemented_mb_function!(wcrtomb, s: i32, wc: i32, ps: i32);
    unimplemented_mb_function!(mbsnrtowcs, dest: i32, src: i32, nms: i32, len: i32, ps: i32);
    unimplemented_mb_function!(mbrtowc, pwc: i32, s: i32, n: i32, ps: i32);
    unimplemented_mb_function!(mbtowc, pwc: i32, s: i32, n: i32);
    unimplemented_mb_function!(mbrlen, s: i32, n: i32, ps: i32);
    unimplemented_mb_function!(mbsrtowcs, dest: i32, src: i32, len: i32, ps: i32);

    #[no_mangle]
    pub unsafe extern "C" fn __mb_cur_max() -> i32 {
        1 // Assuming single-byte encoding
    }
    static mut ERRNO: i32 = 0;

    #[no_mangle]
    pub unsafe extern "C" fn abort() {
        error!("abort called");
        process::abort();
    }

    #[no_mangle]
    pub unsafe extern "C" fn fprintf(_stream: i32, _format: i32, _args: i32) -> i32 {
        error!("fprintf not implemented");
        -1
    }

    #[no_mangle]
    pub unsafe extern "C" fn vfprintf(_stream: i32, _format: i32, _args: i32) -> i32 {
        error!("vfprintf not implemented");
        -1
    }

    #[no_mangle]
    pub unsafe extern "C" fn posix_memalign(memptr: i32, alignment: i32, size: i32) -> i32 {
        // Check if alignment is a power of 2 and >= sizeof(void*)
        if alignment <= 0 || (alignment & (alignment - 1)) != 0 || alignment < 8 {
            return 22; // EINVAL
        }

        // Check for negative size
        if size < 0 {
            return 22; // EINVAL
        }

        // Just use malloc since it already provides aligned memory
        let ptr = malloc(size as usize);
        if ptr.is_null() {
            return 12; // ENOMEM
        }

        // Store the pointer at memptr
        *(memptr as *mut *mut u8) = ptr;

        0 // Success
    }

    #[no_mangle]
    pub unsafe extern "C" fn pthread_mutex_unlock(_mutex: i32) -> i32 {
        0 // Success
    }

    #[no_mangle]
    pub unsafe extern "C" fn fopen(_filename: i32, _mode: i32) -> i32 {
        error!("fopen not implemented");
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn fclose(_stream: i32) -> i32 {
        0 // Success
    }

    #[no_mangle]
    pub unsafe extern "C" fn printf(_format: i32, _args: i32) -> i32 {
        error!("printf not implemented");
        -1
    }

    #[no_mangle]
    pub unsafe extern "C" fn setjmp(_env: i32) -> i32 {
        0 // Initial call return
    }

    #[no_mangle]
    pub unsafe extern "C" fn atoi(_str: i32) -> i32 {
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn __errno() -> i32 {
        &raw mut ERRNO as *mut i32 as i32
    }

    #[no_mangle]
    pub unsafe extern "C" fn modf(x: f64, iptr: i32) -> f64 {
        let int_part = x.trunc();
        *(iptr as *mut f64) = int_part;
        x - int_part
    }

    #[no_mangle]
    pub unsafe extern "C" fn fflush(_stream: i32) -> i32 {
        0 // Success
    }

    #[no_mangle]
    pub unsafe extern "C" fn __assert2(_file: i32, _line: i32, _func: i32, _expr: i32) {
        error!("assertion failed");
        panic!("assertion failed");
    }

    #[no_mangle]
    pub unsafe extern "C" fn asinh(x: f64) -> f64 {
        x.asinh()
    }

    #[no_mangle]
    pub unsafe extern "C" fn getenv(_name: i32) -> i32 {
        0 // NULL
    }

    #[no_mangle]
    pub unsafe extern "C" fn sprintf(_str: i32, _format: i32, _args: i32) -> i32 {
        error!("sprintf not implemented");
        -1
    }

    // File operations
    macro_rules! unimplemented_file_op {
            ($name:ident, $($arg:ident: $type:ty),*) => {
                #[no_mangle]
                pub unsafe extern "C" fn $name($($arg: $type),*) -> i32 {
                    error!(concat!(stringify!($name), " not implemented"));
                    -1
                }
            }
        }

    unimplemented_file_op!(fstat, fd: i32, buf: i32);
    unimplemented_file_op!(chmod, path: i32, mode: i32);
    unimplemented_file_op!(fcntl, fd: i32, cmd: i32, arg: i32);
    unimplemented_file_op!(close, fd: i32);
    unimplemented_file_op!(access, path: i32, mode: i32);
    unimplemented_file_op!(fseek, stream: i32, offset: i32, whence: i32);

    #[no_mangle]
    pub unsafe extern "C" fn fseeko(_stream: i32, _offset: i64, _whence: i32) -> i32 {
        error!("fseeko not implemented");
        -1
    }

    #[no_mangle]
    pub unsafe extern "C" fn ftello(_stream: i32) -> i64 {
        error!("ftello not implemented");
        -1
    }

    #[no_mangle]
    pub unsafe extern "C" fn fwrite(_ptr: i32, _size: i32, _nmemb: i32, _stream: i32) -> i32 {
        error!("fwrite not implemented");
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn fread(_ptr: i32, _size: i32, _nmemb: i32, _stream: i32) -> i32 {
        error!("fread not implemented");
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn sysconf(_name: i32) -> i32 {
        -1
    }

    #[no_mangle]
    pub unsafe extern "C" fn isalpha(c: i32) -> i32 {
        if (c as u8).is_ascii_alphabetic() {
            1
        } else {
            0
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn toupper(c: i32) -> i32 {
        if c >= 'a' as i32 && c <= 'z' as i32 {
            c - 32
        } else {
            c
        }
    }

    // pthread functions
    macro_rules! pthread_stub {
            ($name:ident, $($arg:ident: $type:ty),*) => {
                #[no_mangle]
                pub unsafe extern "C" fn $name($($arg: $type),*) -> i32 {
                    0 // Success
                }
            }
        }

    pthread_stub!(pthread_detach, thread: i32);
    pthread_stub!(pthread_create, thread: i32, attr: i32, start_routine: i32, arg: i32);
    pthread_stub!(pthread_join, thread: i32, retval: i32);
    pthread_stub!(pthread_mutex_init, mutex: i32, attr: i32);
    pthread_stub!(pthread_cond_init, cond: i32, attr: i32);
    pthread_stub!(pthread_mutex_destroy, mutex: i32);
    pthread_stub!(pthread_cond_destroy, cond: i32);
    pthread_stub!(pthread_mutex_lock, mutex: i32);
    pthread_stub!(pthread_cond_broadcast, cond: i32);
    pthread_stub!(pthread_cond_wait, cond: i32, mutex: i32);

    #[no_mangle]
    pub unsafe extern "C" fn pthread_self() -> i32 {
        1 // Dummy thread ID
    }

    #[no_mangle]
    pub unsafe extern "C" fn exit(code: i32) {
        process::exit(code);
    }

    #[no_mangle]
    pub unsafe extern "C" fn clock_gettime(_clk_id: i32, _tp: i32) -> i32 {
        error!("clock_gettime not implemented");
        -1
    }

    // Locale functions
    #[no_mangle]
    pub unsafe extern "C" fn newlocale(_mask: i32, _locale: i32, _base: i32) -> i32 {
        1 // Dummy locale
    }

    #[no_mangle]
    pub unsafe extern "C" fn freelocale(_locale: i32) {
        // No-op
    }

    #[no_mangle]
    pub unsafe extern "C" fn uselocale(_locale: i32) -> i32 {
        1 // Dummy previous locale
    }

    #[no_mangle]
    pub unsafe extern "C" fn vsprintf(_str: i32, _format: i32, _args: i32) -> i32 {
        error!("vsprintf not implemented");
        -1
    }

    #[no_mangle]
    pub unsafe extern "C" fn puts(s: i32) -> i32 {
        // In a real implementation, this would print the null-terminated string at address s
        // For now, just log that it was called and return success
        log::debug!("puts called with pointer: 0x{:x}", s);
        1 // Return positive number on success as per C standard
    }

    #[no_mangle]
    pub unsafe extern "C" fn wasm_js_console_log_trace(the_str: *const i8) {
        let c_str = CStr::from_ptr(the_str);
        if let Ok(rust_str) = c_str.to_str() {
            trace!("{}", rust_str);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn wasm_js_console_log_info(the_str: *const i8) {
        let c_str = CStr::from_ptr(the_str);
        if let Ok(rust_str) = c_str.to_str() {
            info!("{}", rust_str);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn wasm_js_console_log_warn(the_str: *const i8) {
        let c_str = CStr::from_ptr(the_str);
        if let Ok(rust_str) = c_str.to_str() {
            warn!("{}", rust_str);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn wasm_js_console_log_debug(the_str: *const i8) {
        let c_str = CStr::from_ptr(the_str);
        if let Ok(rust_str) = c_str.to_str() {
            debug!("{}", rust_str);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn wasm_js_console_log_error(the_str: *const i8) {
        let c_str = CStr::from_ptr(the_str);
        if let Ok(rust_str) = c_str.to_str() {
            error!("{}", rust_str);
        }
    }
}
