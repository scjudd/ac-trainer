use crate::proc;

pub fn list(handle: proc::Handle) -> Result<Vec<Player>, String> {
    let list_addr = unsafe { *(proc::read(handle, 0x50f4f8, 4)?.as_ptr() as *const u32) };

    let list_length =
        unsafe { *(proc::read(handle, 0x50f500, 4)?.as_ptr() as *const u32) as usize };

    let mut list = Vec::with_capacity(list_length);

    for index in 0..list_length {
        let entity_addr = unsafe {
            *(proc::read(handle, list_addr + (index as u32 * 0x4), 4)?.as_ptr() as *const u32)
        };

        // When entities are removed, their entity list pointer is set to null, but the remaining
        // entities are not moved.
        if entity_addr == 0 {
            continue;
        }

        let player = Player::read(handle, entity_addr);
        list.push(player);
    }

    Ok(list)
}

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub health: i32,
    pub armor: i32,
    pub name: String,
}

// This isn't the actual size of the playerent struct in AC, but rather the number of bytes that
// must be read from the remote process to populate a local Player struct.
pub const PLAYER_SIZE: usize = 564;

pub const PLAYER_X_OFFSET: isize = 0x4;
pub const PLAYER_Y_OFFSET: isize = 0x8;
pub const PLAYER_Z_OFFSET: isize = 0xC;
pub const PLAYER_YAW_OFFSET: isize = 0x40;
pub const PLAYER_PITCH_OFFSET: isize = 0x44;
pub const PLAYER_HEALTH_OFFSET: isize = 0xF8;
pub const PLAYER_ARMOR_OFFSET: isize = 0xFC;

// The player name is a char[16], but we'll represent it as a native Rust string for ease of use.
pub const PLAYER_NAME_OFFSET: isize = 0x225;
pub const PLAYER_NAME_SIZE: usize = 16;

impl Player {
    pub fn read(handle: proc::Handle, addr: u32) -> Player {
        let raw = proc::read(handle, addr, PLAYER_SIZE).expect("failed to read player entity");

        let (x, y, z, health, armor) = unsafe {
            (
                *(raw.as_ptr().offset(PLAYER_X_OFFSET) as *const f32),
                *(raw.as_ptr().offset(PLAYER_Y_OFFSET) as *const f32),
                *(raw.as_ptr().offset(PLAYER_Z_OFFSET) as *const f32),
                *(raw.as_ptr().offset(PLAYER_HEALTH_OFFSET) as *const i32),
                *(raw.as_ptr().offset(PLAYER_ARMOR_OFFSET) as *const i32),
            )
        };

        let name_start = PLAYER_NAME_OFFSET as usize;
        let name_end = PLAYER_NAME_OFFSET as usize + PLAYER_NAME_SIZE - 1;
        let name_bytes = raw[name_start..name_end]
            .iter()
            .take_while(|&c| *c != 0)
            .copied()
            .collect::<Vec<u8>>();
        let name = String::from_utf8(name_bytes).expect("invalid utf8 data in player name string");

        Player {
            x,
            y,
            z,
            health,
            armor,
            name,
        }
    }
}
