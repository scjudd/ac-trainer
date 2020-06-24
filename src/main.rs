mod entities;
mod proc;
mod winapi;

fn read_player(handle: proc::Handle, addr: u32) -> entities::Player {
    let raw =
        proc::read(handle, addr, entities::PLAYER_SIZE).expect("failed to read player entity");

    let (x, y, z, health, armor) = unsafe {
        (
            *(raw.as_ptr().offset(entities::PLAYER_X_OFFSET) as *const f32),
            *(raw.as_ptr().offset(entities::PLAYER_Y_OFFSET) as *const f32),
            *(raw.as_ptr().offset(entities::PLAYER_Z_OFFSET) as *const f32),
            *(raw.as_ptr().offset(entities::PLAYER_HEALTH_OFFSET) as *const i32),
            *(raw.as_ptr().offset(entities::PLAYER_ARMOR_OFFSET) as *const i32),
        )
    };

    let name_start = entities::PLAYER_NAME_OFFSET as usize;
    let name_end = entities::PLAYER_NAME_OFFSET as usize + entities::PLAYER_NAME_SIZE - 1;
    let name_bytes = raw[name_start..name_end]
        .iter()
        .take_while(|&c| *c != 0)
        .copied()
        .collect::<Vec<u8>>();
    let name = String::from_utf8(name_bytes).expect("invalid utf8 data in player name string");

    entities::Player {
        x,
        y,
        z,
        health,
        armor,
        name,
    }
}

fn print_header() {
    println!("Addr        Name              Health  Armor  X           Y           Z");
    println!("===============================================================================");
}

fn print_player(entity_addr: u32, player: &entities::Player) {
    println!(
        "{:<#10x}  {:16}  {:<4}    {:<4}   {:<10}  {:<10}  {:<10}",
        entity_addr, player.name, player.health, player.armor, player.x, player.y, player.z,
    );
}

fn run_once(handle: proc::Handle) {
    print_header();

    // The current player

    let entity_addr = unsafe { *(proc::read(handle, 0x50f4f4, 4).unwrap().as_ptr() as *const u32) };
    let player = read_player(handle, entity_addr);
    print_player(entity_addr, &player);

    // All other players

    let entity_list_addr =
        unsafe { *(proc::read(handle, 0x50f4f8, 4).unwrap().as_ptr() as *const u32) };

    let entity_list_length =
        unsafe { *(proc::read(handle, 0x50f500, 4).unwrap().as_ptr() as *const u32) as usize };

    for index in 0..entity_list_length {

        let entity_addr = unsafe {
            *(proc::read(handle, entity_list_addr + (index as u32 * 0x4), 4)
                .unwrap()
                .as_ptr() as *const u32)
        };

        // When entities are removed, their entity list pointer is set to null, but the remaining
        // entities are not moved.
        if entity_addr == 0 {
            continue;
        }

        let player = read_player(handle, entity_addr);
        print_player(entity_addr, &player);
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

        let handle = proc::open(pid).expect("failed to open process");

        while proc::still_active(handle).expect("failed to check process exit code") {
            run_once(handle);
        }

        proc::close(handle).expect("failed to close process");

        eprintln!("Game closed.");
    }
}
