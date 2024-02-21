#![feature(lazy_cell)]

use crate::d2::reveal::LevelId;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock, Mutex,
};
use windows::Win32::{
    Foundation::{BOOL, HINSTANCE},
    System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    UI::Input::KeyboardAndMouse::VK_DELETE,
};

mod d2;
mod offsets;
mod utils;

static EXITING: AtomicBool = AtomicBool::new(false);

#[no_mangle]
extern "stdcall" fn DllMain(
    hinstance: HINSTANCE,
    reason: u32,
    _reserved: *mut std::ffi::c_void,
) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            std::thread::spawn(move || main_thread(hinstance));
        }
        DLL_PROCESS_DETACH => {
            #[cfg(debug_assertions)]
            utils::free_console();
        }
        _ => {}
    };

    BOOL::from(true)
}

static REVEALED_AREAS: LazyLock<Mutex<Vec<LevelId>>> = LazyLock::new(|| Mutex::new(Vec::new()));

fn reset_revealed_areas() {
    let mut revealed_areas = REVEALED_AREAS.lock().unwrap();
    revealed_areas.clear();
}

fn on_loop() {
    let player = d2::unit::Unit::get_local_player();
    if let Some(player) = player {
        // mean's we're in a game
        if player.is_valid() {
            let level = player.get_level();
            if let Some(level) = level {
                let id = level.level_no;
                let mut revealed_areas = REVEALED_AREAS.lock().unwrap();
                if !revealed_areas.contains(&id) {
                    println!("[+] revealing area: {:?}", id);
                    level.reveal();
                    revealed_areas.push(id);
                }
            }
        }
    } else if !REVEALED_AREAS.lock().unwrap().is_empty() {
        println!("[+] out of game, resetting revealed areas");
        reset_revealed_areas();
    }

    if utils::key_released(VK_DELETE.0 as i32) {
        EXITING.store(true, Ordering::Relaxed);
        utils::unload();
    }

    // not really necessary to hammer the CPU for revealing map stuff
    std::thread::sleep(std::time::Duration::from_millis(100));
}

fn main_thread(_hinstance: HINSTANCE) {
    #[cfg(debug_assertions)]
    utils::alloc_console();

    println!("[-] successfully loaded d2rmh.dll");

    while !EXITING.load(Ordering::SeqCst) {
        on_loop();
    }
}
