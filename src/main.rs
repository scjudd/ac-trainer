mod aimbot;
mod code;
mod entities;
mod proc;
mod winapi;

use entities::Player;
use proc::Read;

pub fn capslock_enabled() -> bool {
    unsafe { winapi::GetKeyState(winapi::VK_CAPITAL) & 1 == 1 }
}

fn main() {
    loop {
        let pid = loop {
            match proc::find("ac_client.exe") {
                Some(pid) => break pid,
                None => {
                    eprintln!("Waiting for game to launch...");
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    continue;
                }
            }
        };

        aimbot::spawn_thread(pid);

        let handle = proc::open(pid).expect("failed to open process");

        code::godmode()
            .inject(handle)
            .expect("failed to inject godmode hook")
            .enable(handle)
            .expect("failed to enable godmode hook");

        while proc::still_active(handle).expect("failed to check process exit code") {
            run_once(handle);
        }

        proc::close(handle).expect("failed to close process");

        eprintln!("Game closed.");
    }
}

fn run_once(handle: proc::Handle) {
    print_header();

    let my_addr = u32::read(handle, 0x50f4f4).expect("failed to read player pointer");
    let me = Player::read(handle, my_addr).expect("failed to read player entity");
    print_player(&me);

    let players = entities::player_list(handle).expect("failed to read player list");
    for player in players {
        print_player(&player);
    }

    println!("");
    std::thread::sleep(std::time::Duration::from_millis(1000));
}

fn print_header() {
    println!("Name              Health  Armor  X           Y           Z");
    println!("===================================================================");
}

fn print_player(player: &Player) {
    println!(
        "{:16}  {:<4}    {:<4}   {:<10}  {:<10}  {:<10}",
        player.name, player.health, player.armor, player.x, player.y, player.z,
    );
}
