use crate::capslock_enabled;
use crate::entities::{self, Player};
use crate::proc::{self, Read, Write};

struct Angle {
    yaw: f32,
    pitch: f32,
}

pub fn spawn_thread(pid: proc::Pid) {
    std::thread::spawn(move || run(pid));
}

fn run(pid: proc::Pid) {
    let handle = proc::open(pid).expect("failed to open process");

    while proc::still_active(handle).expect("failed to check process exit code") {
        run_once(handle);
    }

    proc::close(handle).expect("failed to close process");
}

fn run_once(handle: proc::Handle) {
    if !capslock_enabled() {
        std::thread::sleep(std::time::Duration::from_millis(100));
        return;
    }

    let my_addr = u32::read(handle, 0x50f4f4).expect("failed to read player pointer");
    let me = Player::read(handle, my_addr).expect("failed to read player entity");

    // Don't aim while dead: it's awkward.
    if me.health <= 0 {
        return;
    }

    let players = entities::player_list(handle).expect("failed to read player list");
    if let Some(target_player) = closest_living(&me, &players) {
        let angle = calc_angle(&me, target_player);
        aim(handle, my_addr, &angle);
    }
}

fn closest_living<'a>(me: &Player, players: &'a Vec<Player>) -> Option<&'a Player> {
    players
        .iter()
        .filter(|p| p.health > 0)
        .min_by(|x, y| distance(me, x).partial_cmp(&distance(me, y)).unwrap())
}

fn distance(src: &Player, dst: &Player) -> f32 {
    let (x, y, z) = (dst.x - src.x, dst.y - src.y, dst.z - src.z);
    let base = (x.powi(2) + y.powi(2)).sqrt();
    (base.powi(2) + z.powi(2)).sqrt()
}

fn calc_angle(src: &Player, dst: &Player) -> Angle {
    let (x, y, z) = (dst.x - src.x, dst.y - src.y, dst.z - src.z);
    let base = (x.powi(2) + y.powi(2)).sqrt();

    let mut yaw = y.atan2(x).to_degrees() + 90.0;
    if yaw <= 0.0 {
        yaw += 360.0;
    }

    let pitch = (z / base).atan().to_degrees();

    Angle { yaw, pitch }
}

fn aim(handle: proc::Handle, player_addr: u32, angle: &Angle) {
    angle
        .yaw
        .write(handle, player_addr + entities::PLAYER_YAW_OFFSET as u32)
        .expect("failed to set player yaw");

    angle
        .pitch
        .write(handle, player_addr + entities::PLAYER_PITCH_OFFSET as u32)
        .expect("failed to set player pitch");
}
