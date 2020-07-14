use crate::proc::{self, Read};

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub health: i32,
    pub armor: i32,
    pub name: String,
}

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

impl proc::Read for Player {
    fn read(handle: proc::Handle, addr: u32) -> Result<Player, String> {
        let x = f32::read(handle, addr + PLAYER_X_OFFSET as u32)?;
        let y = f32::read(handle, addr + PLAYER_Y_OFFSET as u32)?;
        let z = f32::read(handle, addr + PLAYER_Z_OFFSET as u32)?;
        let health = i32::read(handle, addr + PLAYER_HEALTH_OFFSET as u32)?;
        let armor = i32::read(handle, addr + PLAYER_ARMOR_OFFSET as u32)?;

        let name_bytes = proc::read(handle, addr + PLAYER_NAME_OFFSET as u32, PLAYER_NAME_SIZE)?
            .iter()
            .take_while(|&c| *c != 0)
            .copied()
            .collect::<Vec<u8>>();
        let name = String::from_utf8(name_bytes)
            .map_err(|_| String::from("invalid utf8 data in player name string"))?;

        Ok(Player {
            x,
            y,
            z,
            health,
            armor,
            name,
        })
    }
}

pub fn player_list(handle: proc::Handle) -> Result<Vec<Player>, String> {
    let list_addr = u32::read(handle, 0x50f4f8)?;
    let list_length = u32::read(handle, 0x50f500)? as usize;
    let mut list = Vec::with_capacity(list_length);

    for index in 0..list_length {
        let player_addr = u32::read(handle, list_addr + (index as u32 * 0x4))?;

        // When entities are removed, their entity list pointer is set to null, but the remaining
        // entities are not moved.
        if player_addr == 0 {
            continue;
        }

        let player = Player::read(handle, player_addr)?;
        list.push(player);
    }

    Ok(list)
}
