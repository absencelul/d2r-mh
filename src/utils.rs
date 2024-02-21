use std::{
    ffi::CString,
    sync::{LazyLock, Mutex, OnceLock},
};
#[cfg(debug_assertions)]
use windows::Win32::System::Console::{AllocConsole, FreeConsole};
use windows::{
    core::PCSTR,
    Win32::{
        System::LibraryLoader::{FreeLibraryAndExitThread, GetModuleHandleA},
        UI::Input::KeyboardAndMouse::GetAsyncKeyState,
    },
};

#[cfg(debug_assertions)]
pub fn alloc_console() {
    unsafe { AllocConsole().expect("Failed to allocate console") };
}

#[cfg(debug_assertions)]
pub fn free_console() {
    unsafe { FreeConsole().expect("Failed to free console") };
}

pub fn unload() {
    let module_name = CString::new("d2rmh.dll").unwrap();

    println!("[-] unloading {:?}", module_name);

    #[cfg(debug_assertions)]
    free_console();

    unsafe {
        let module_handle =
            GetModuleHandleA(PCSTR::from_raw(module_name.as_ptr() as *const u8)).unwrap();
        FreeLibraryAndExitThread(module_handle, 0);
    }
}

static PRESSED_KEYS: LazyLock<Mutex<[bool; 255]>> = LazyLock::new(|| Mutex::new([false; 255]));

pub fn key_released(key: i32) -> bool {
    let mut pressed_keys = PRESSED_KEYS.lock().unwrap();
    let result = unsafe { GetAsyncKeyState(key) };
    let pressed = (result >> 15) & 1 != 0;
    let was_pressed = pressed_keys[key as usize];
    pressed_keys[key as usize] = pressed;
    !pressed && was_pressed
}

static BASE_ADDRESS: OnceLock<usize> = OnceLock::new();

pub fn get_base_address() -> usize {
    *BASE_ADDRESS
        .get_or_init(|| unsafe { GetModuleHandleA(PCSTR(std::ptr::null())).unwrap().0 as usize })
}
