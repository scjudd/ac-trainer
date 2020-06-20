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
pub const PLAYER_HEALTH_OFFSET: isize = 0xF8;
pub const PLAYER_ARMOR_OFFSET: isize = 0xFC;

// The player name is a char[16], but we'll represent it as a native Rust string for ease of use.
pub const PLAYER_NAME_OFFSET: isize = 0x225;
pub const PLAYER_NAME_SIZE: usize = 16;
