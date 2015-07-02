use std::thread::sleep_ms;

extern crate demonplayer;

use demonplayer::Demonplayer;

fn main() {
    let mut player = Demonplayer::new()
                 .unwrap_or_else(|e| {
                     panic!("demonplayer init failed: {}", e);
                 });
    player.print_info();
    
    let _ = player.play();
    
    loop {
        sleep_ms(1000);    
    }
}