mod aimbot;
mod entities;
mod proc;
mod winapi;

fn print_header() {
    println!("Name              Health  Armor  X           Y           Z");
    println!("===================================================================");
}

fn print_player(player: &entities::Player) {
    println!(
        "{:16}  {:<4}    {:<4}   {:<10}  {:<10}  {:<10}",
        player.name, player.health, player.armor, player.x, player.y, player.z,
    );
}

fn run_once(handle: proc::Handle) {
    print_header();

    // The current player

    let my_addr = unsafe { *(proc::read(handle, 0x50f4f4, 4).unwrap().as_ptr() as *const u32) };
    let me = entities::Player::read(handle, my_addr);
    print_player(&me);

    // All other players

    let players = entities::list(handle).expect("failed to read entity list");
    for player in players {
        print_player(&player);
    }

    println!("");
    std::thread::sleep(std::time::Duration::from_millis(1000));
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

        while proc::still_active(handle).expect("failed to check process exit code") {
            run_once(handle);
        }

        proc::close(handle).expect("failed to close process");

        eprintln!("Game closed.");
    }
}
