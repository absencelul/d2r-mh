use super::reveal::{Act, Level, Room};
use crate::{offsets, utils};

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnitType {
    Player = 0u32,
    Monster,
    Object,
    Missile,
    Item,
    Tile,
    Exit,
    Xy,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PlayerPath {
    pub offset_x: u16,
    pub x: u16,
    pub offset_y: u16,
    pub y: u16,
    pub target_x: u32,
    pub target_y: u32,
    pad_0010: [u8; 16],
    pub room: *const Room,
}

#[repr(C)]
#[allow(unused)]
#[derive(Copy, Clone)]
pub struct Unit {
    pub unit_type: UnitType,     // 0x0000
    pub class_id: u32,           // 0x0004
    pub unit_id: u32,            // 0x0008
    pub anim_mode: u32,          // 0x000C
    pub unit_data: usize,        // 0x0010
    pub act_id: u64,             // 0x0018
    pub act: *const Act,         // 0x0020
    pub low_seed: u32,           // 0x0028
    pub high_seed: u32,          // 0x002C
    pad_0030: [u8; 8],           // 0x0030
    pub path: *const PlayerPath, // 0x0038
    anim_seq: *const u64,        // 0x0040
    pub seq_frame_count: u32,    // 0x0048
    pub seq_frame: u32,          // 0x004C
    pad_0050: [u8; 256],         // 0x0050
    pub list_next: *const Unit,  // 0x0150
    pub room_next: *const Unit,  // 0x0158
    pad_0160: [u8; 24],          // 0x0160
    size_x: u32,                 // 0x0178
    size_y: u32,                 // 0x017C
}

type GetPlayerUnitFn = extern "fastcall" fn(u32) -> *mut Unit;

impl Unit {
    pub fn get_local_player<'a>() -> Option<&'a mut Unit> {
        let idx = unsafe {
            ((utils::get_base_address() + offsets::PLAYER_UNIT_IDX) as *const u32)
                .as_ref()
                .map(|&x| x)
        };
        unsafe {
            std::mem::transmute::<usize, GetPlayerUnitFn>(
                utils::get_base_address() + offsets::GET_PLAYER_UNIT,
            )(idx?)
            .as_mut()
        }
    }

    pub fn is_valid(&self) -> bool {
        // only care about players for this
        if self.unit_id == u32::MAX || self.unit_type != UnitType::Player {
            return false;
        }
        true
    }

    pub fn get_level<'a>(&self) -> Option<&'a Level> {
        unsafe { self.path.as_ref()?.room.as_ref()?.room_ex.as_ref()?.level.as_ref() }
    }
}
