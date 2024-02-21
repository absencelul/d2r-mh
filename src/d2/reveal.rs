use crate::{offsets, utils};

#[repr(C)]
pub struct Room {
    pub rooms_near: *const *const Room, // 0x0000
    pad_0008: [u8; 16],                 // 0x0008
    pub room_ex: *mut RoomEx,           // 0x0018
    pad_0020: [u8; 216],                // 0x0020
}

type LoadActIFn = extern "fastcall" fn(*const Room) -> *const std::ffi::c_void;

impl Room {
    fn reveal(&self) {
        unsafe {
            std::mem::transmute::<usize, LoadActIFn>(utils::get_base_address() + offsets::LOAD_ACT_I)(self as *const Room);
        }
    }
}

#[repr(C)]
pub struct RoomEx {
    pad_0000: [u8; 16],                     // 0x0000
    pub room_ex_near: *const *const RoomEx, // 0x0010
    pub rooms_near: u32,                    // 0x0018
    pad_001c: [u8; 28],                     // 0x001C
    pub room_ex_first: *const RoomEx,       // 0x0038
    pad_0040: [u8; 8],                      // 0x0040
    pub room_ex_next: *const RoomEx,        // 0x0048
    room_flags: u32,                        // 0x0050
    pad_0054: [u8; 4],                      // 0x0054
    pub room: *const Room,                  // 0x0058
    pos_x: u32,                             // 0x0060
    pos_y: u32,                             // 0x0064
    size_x: u32,                            // 0x0068
    size_y: u32,                            // 0x006C
    pad_0070: [u8; 32],                     // 0x0070
    pub level: *const Level,                // 0x0090
    preset: *const usize,                   // 0x0098
}

type AddRoomDataFn = extern "fastcall" fn(*const Act, LevelId, u32, u32, *const Room) -> *const std::ffi::c_void;
type RemoveRoomDataFn = extern "fastcall" fn(*const Act, LevelId, u32, u32, *const Room) -> *const std::ffi::c_void;

impl RoomEx {
    fn add_room_data(&self, act: &Act, level_id: LevelId) {
        unsafe {
            std::mem::transmute::<usize, AddRoomDataFn>(
                utils::get_base_address() + offsets::ADD_ROOM_DATA,
            )(act, level_id, self.pos_x, self.pos_y, self.room);
        }
    }

    fn remove_room_data(&self, act: &Act, level_id: LevelId) {
        unsafe {
            std::mem::transmute::<usize, RemoveRoomDataFn>(
                utils::get_base_address() + offsets::REMOVE_ROOM_DATA,
            )(act, level_id, self.pos_x, self.pos_y, self.room);
        }
    }

    fn is_initialized(&self) -> bool {
        !self.room.is_null()
    }

    fn init(&self, act: &Act, level_id: LevelId) -> bool {
        if !self.is_initialized() {
            self.add_room_data(act, level_id);
            return true;
        }
        false
    }

    fn cleanup(&self, act: &Act, level_id: LevelId) {
        if self.is_initialized() {
            self.remove_room_data(act, level_id);
        }
    }

    fn reveal(&self, act: &Act) {
        let level_id = unsafe { self.level.as_ref() }.unwrap().level_no;
        let should_cleanup = self.init(act, level_id);
        if let Some(room) = unsafe { self.room.as_ref() } {
            if self.is_initialized() {
                room.reveal();
                if should_cleanup {
                    self.cleanup(act, level_id);
                }
            }
        }
    }
}

#[repr(C)]
#[allow(unused)]
pub struct Level {
    pad_0000: [u8; 16],             // 0x0000
    pub room_ex_first: *mut RoomEx, // 0x0010
    pad_0018: [u8; 16],             // 0x0018
    pub pos_x: u32,                 // 0x0028
    pub pos_y: u32,                 // 0x002C
    pub size_x: u32,                // 0x0030
    pub size_y: u32,                // 0x0034
    pad_0038: [u8; 384],            // 0x0038
    pub level_next: *mut Level,     // 0x01B8
    pad_01c0: [u8; 8],              // 0x01C0
    pub act_misc: *mut ActMisc,     // 0x01C8
    pad_01d0: [u8; 40],             // 0x01D0
    pub level_no: LevelId,          // 0x01F8
    pad_01fc: [u8; 140],            // 0x01FC
}

type InitLevelFn = extern "fastcall" fn(*const Level) -> *const std::ffi::c_void;

impl Level {
    fn is_initialized(&self) -> bool {
        !self.room_ex_first.is_null()
    }

    fn init(&self) {
        if !self.is_initialized() {
            unsafe {
                std::mem::transmute::<usize, InitLevelFn>(utils::get_base_address() + offsets::INIT_LEVEL)(
                    self as *const Level,
                );
            }
        }
    }

    fn process(&self, room_ex: &RoomEx, act: &Act) {
        room_ex.reveal(act);
        let rooms_near = unsafe {
            std::slice::from_raw_parts(room_ex.room_ex_near, room_ex.rooms_near as usize)
        };
        for &room_ex_near_ptr in rooms_near {
            let room_ex_near = match unsafe { room_ex_near_ptr.as_ref() } {
                Some(room_ex_near) => room_ex_near,
                None => continue,
            };
            let level_near = match unsafe { room_ex_near.level.as_ref() } {
                Some(level_near) => level_near,
                None => continue,
            };
            if self.level_no == level_near.level_no || !level_near.is_initialized() {
                continue;
            }
            let act = match unsafe { level_near.act_misc.as_ref() } {
                Some(act_misc) => act_misc.act,
                None => continue,
            };
            let act = match unsafe { act.as_ref() } {
                Some(act) => act,
                None => continue,
            };
            room_ex_near.reveal(act);
        }
    }

    pub fn reveal(&self) {
        self.init();
        if self.is_initialized() {
            let act = match unsafe { self.act_misc.as_ref() } {
                Some(act_misc) => act_misc.act,
                None => return,
            };
            let act = match unsafe { act.as_ref() } {
                Some(act) => act,
                None => return,
            };
            let mut room_ex_opts = unsafe { self.room_ex_first.as_ref() };
            while let Some(room_ex) = room_ex_opts {
                self.process(room_ex, act);
                unsafe {
                    room_ex_opts = room_ex.room_ex_next.as_ref();
                }
            }
        }
    }
}

#[repr(C)]
pub struct Act {
    pad_0000: [u8; 32],     // 0x0000
    pub room: *mut Room,    // 0x0020
    pub act_no: u32,        // 0x0028
    pad_002c: [u8; 76],     // 0x002C
    act_misc: *mut ActMisc, // 0x0078
    pad_0080: [u8; 16],     // 0x0080
}

impl Act {
    pub fn reveal(&self) {
        let mut level_opts = unsafe { self.act_misc.as_ref() }
            .and_then(|act_misc| unsafe { act_misc.level_first.as_ref() });
        while let Some(level) = level_opts {
            level.reveal();
            unsafe {
                level_opts = level.level_next.as_ref();
            }
        }
    }
}

#[repr(C)]
pub struct ActMisc {
    pad_0000: [u8; 2144],        // 0x0000
    pub act: *mut Act,           // 0x0A30
    pad_0a38: [u8; 8],           // 0x0A38
    pub level_first: *mut Level, // 0x0A40
    act_no: u32,                 // 0x0A48
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
pub enum LevelId {
    None = 0,
    RogueEncampment = 1,
    BloodMoor = 2,
    ColdPlains = 3,
    StonyField = 4,
    DarkWood = 5,
    BlackMarsh = 6,
    TamoeHighland = 7,
    DenOfEvil = 8,
    CaveLevel1 = 9,
    UndergroundPassageLevel1 = 10,
    HoleLevel1 = 11,
    PitLevel1 = 12,
    CaveLevel2 = 13,
    UndergroundPassageLevel2 = 14,
    HoleLevel2 = 15,
    PitLevel2 = 16,
    BurialGrounds = 17,
    Crypt = 18,
    Mausoleum = 19,
    ForgottenTower = 20,
    TowerCellarLevel1 = 21,
    TowerCellarLevel2 = 22,
    TowerCellarLevel3 = 23,
    TowerCellarLevel4 = 24,
    TowerCellarLevel5 = 25,
    MonasteryGate = 26,
    OuterCloister = 27,
    Barracks = 28,
    JailLevel1 = 29,
    JailLevel2 = 30,
    JailLevel3 = 31,
    InnerCloister = 32,
    Cathedral = 33,
    CatacombsLevel1 = 34,
    CatacombsLevel2 = 35,
    CatacombsLevel3 = 36,
    CatacombsLevel4 = 37,
    Tristram = 38,
    MooMooFarm = 39,
    LutGholein = 40,
    RockyWaste = 41,
    DryHills = 42,
    FarOasis = 43,
    LostCity = 44,
    ValleyOfSnakes = 45,
    CanyonOfTheMagi = 46,
    A2SewersLevel1 = 47,
    A2SewersLevel2 = 48,
    A2SewersLevel3 = 49,
    HaremLevel1 = 50,
    HaremLevel2 = 51,
    PalaceCellarLevel1 = 52,
    PalaceCellarLevel2 = 53,
    PalaceCellarLevel3 = 54,
    StonyTombLevel1 = 55,
    HallsOfTheDeadLevel1 = 56,
    HallsOfTheDeadLevel2 = 57,
    ClawViperTempleLevel1 = 58,
    StonyTombLevel2 = 59,
    HallsOfTheDeadLevel3 = 60,
    ClawViperTempleLevel2 = 61,
    MaggotLairLevel1 = 62,
    MaggotLairLevel2 = 63,
    MaggotLairLevel3 = 64,
    AncientTunnels = 65,
    TalRashasTomblevel1 = 66,
    TalRashasTomblevel2 = 67,
    TalRashasTomblevel3 = 68,
    TalRashasTomblevel4 = 69,
    TalRashasTomblevel5 = 70,
    TalRashasTomblevel6 = 71,
    TalRashasTomblevel7 = 72,
    DurielsLair = 73,
    ArcaneSanctuary = 74,
    KurastDocktown = 75,
    SpiderForest = 76,
    GreatMarsh = 77,
    FlayerJungle = 78,
    LowerKurast = 79,
    KurastBazaar = 80,
    UpperKurast = 81,
    KurastCauseway = 82,
    Travincal = 83,
    SpiderCave = 84,
    SpiderCavern = 85,
    SwampyPitLevel1 = 86,
    SwampyPitLevel2 = 87,
    FlayerDungeonLevel1 = 88,
    FlayerDungeonLevel2 = 89,
    SwampyPitLevel3 = 90,
    FlayerDungeonLevel3 = 91,
    A3SewersLevel1 = 92,
    A3SewersLevel2 = 93,
    RuinedTemple = 94,
    DisusedFane = 95,
    ForgottenReliquary = 96,
    ForgottenTemple = 97,
    RuinedFane = 98,
    DisusedReliquary = 99,
    DuranceOfHateLevel1 = 100,
    DuranceOfHateLevel2 = 101,
    DuranceOfHateLevel3 = 102,
    ThePandemoniumFortress = 103,
    OuterSteppes = 104,
    PlainsOfDespair = 105,
    CityOfTheDamned = 106,
    RiverOfFlame = 107,
    ChaosSanctuary = 108,
    Harrogath = 109,
    BloodyFoothills = 110,
    FrigidHighlands = 111,
    ArreatPlateau = 112,
    CrystalizedPassage = 113,
    FrozenRiver = 114,
    GlacialTrail = 115,
    DrifterCavern = 116,
    FrozenTundra = 117,
    AncientsWay = 118,
    IcyCellar = 119,
    ArreatSummit = 120,
    NihlathaksTemple = 121,
    HallsOfAnguish = 122,
    HallsOfPain = 123,
    HallsOfVaught = 124,
    Abaddon = 125,
    PitOfAcheron = 126,
    InfernalPit = 127,
    TheWorldstoneKeepLevel1 = 128,
    TheWorldstoneKeepLevel2 = 129,
    TheWorldstoneKeepLevel3 = 130,
    ThroneOfDestruction = 131,
    TheWorldstoneChamber = 132,
    MatronsDen = 133,
    FogottenSands = 134,
    FurnaceOfPain = 135,
    Tristram2 = 136,
}

impl LevelId {
    pub fn is_town(&self) -> bool {
        matches!(
            self,
            Self::RogueEncampment
                | Self::LutGholein
                | Self::KurastDocktown
                | Self::ThePandemoniumFortress
                | Self::Harrogath
                | Self::None
        )
    }
}
