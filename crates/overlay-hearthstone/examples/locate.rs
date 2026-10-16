//! Prints where the running Hearthstone keeps its logs and log settings:
//!
//!     cargo run -p overlay-hearthstone --example locate

use overlay_games::{Catalog, Platform, list_processes};

fn main() {
    let running = Catalog::builtin().detect(Platform::CURRENT, &list_processes());
    let Some(game) = running.iter().find(|g| g.game_id == "hearthstone") else {
        println!("Hearthstone is not running");
        return;
    };
    println!("pid {} ({:?})", game.pid, game.flavor);
    match overlay_hearthstone::locate::locate(game.pid, game.flavor) {
        Some(install) => {
            println!("logs:       {}", install.logs_dir.display());
            println!("log.config: {}", install.log_config.display());
            println!(
                "Power.log:  {}",
                overlay_hearthstone::locate::newest_power_log(&install.logs_dir).display()
            );
        }
        None => println!("not found"),
    }
}
