use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyValue {
    key: *const c_char,
    value: *const c_char,
}

#[repr(C)]
pub struct CommandData {
    command: *const c_char,
    param1: f64,
    param2: f64,
}

#[unsafe(no_mangle)]
pub extern "C" fn Init(path: *const c_char) {
    if path.is_null() {
        return;
    }

    unsafe {
        let c_str = CStr::from_ptr(path);
        if let Ok(rust_str) = c_str.to_str() {
            println!("[NSR] Init called with path: {}", rust_str);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Render(pairs: *const KeyValue, count: usize, out_len: &mut usize) -> *mut u8 {
    if pairs.is_null() || count == 0 {
        *out_len = 0;
        return ptr::null_mut();
    }

    let mut state = std::collections::HashMap::new();

    unsafe {
        for i in 0..count {
            let item = &*pairs.add(i);
            let key = CStr::from_ptr(item.key).to_string_lossy().into_owned();
            let value = CStr::from_ptr(item.value).to_string_lossy().into_owned();
            state.insert(key, value);
        }
    }

    // Parse height, width, and format from the state
    let width: usize = match state.get("texW").and_then(|v| v.parse().ok()) {
        Some(w) => w,
        None => {
            *out_len = 0;
            return ptr::null_mut();
        }
    };

    let height: usize = match state.get("texH").and_then(|v| v.parse().ok()) {
        Some(h) => h,
        None => {
            *out_len = 0;
            return ptr::null_mut();
        }
    };

    let format = state.get("texFormat").map(|v| v.to_uppercase()).unwrap_or("RGBA".to_string());

    let channels = match format.as_str() {
        "RGB" => 3,
        "RGBA" => 4,
        _ => {
            *out_len = 0;
            return ptr::null_mut(); // Nieobsługiwany format
        }
    };

    let total_size = width * height * channels;
    *out_len = total_size;

    let buf = unsafe { libc::malloc(total_size) as *mut u8 };
    if buf.is_null() {
        *out_len = 0;
        return ptr::null_mut();
    }

    unsafe {
        for y in 0..height {
            for x in 0..width {
                let offset = (y * width + x) * channels;
                for c in 0..channels {
                    *buf.add(offset + c) = ((x + y + c) % 256) as u8; // przykładowe dane
                }
            }
        }
    }

    buf
}


#[unsafe(no_mangle)]
pub extern "C" fn GetCommands(count: &mut usize) -> *mut CommandData {
    let commands: Vec<(&str, f64, f64)> = vec![]; // <--- tu lista komend (pusta)

    *count = commands.len();
    let mut out_vec = Vec::with_capacity(*count);

    for (cmd, p1, p2) in commands {
        let cmd_c = CString::new(cmd).unwrap().into_raw();
        out_vec.push(CommandData {
            command: cmd_c,
            param1: p1,
            param2: p2,
        });
    }

    let ptr = out_vec.as_mut_ptr();
    std::mem::forget(out_vec); // nie zwalniaj wektora – użytkownik musi zwolnić ręcznie
    ptr
}