use std::io;
use std::thread::sleep_ms;

extern crate demonplayer;

use demonplayer::Demonplayer;

fn wait_for_line(prompt: &str) {
    let mut reader = io::stdin();
    println!("{}", prompt);
    let mut line_buf = "".to_string();
    let _ = reader.read_line(&mut line_buf).unwrap();
}

fn main() {
    let mut player = Demonplayer::new()
                 .unwrap_or_else(|e| {
                     panic!("demonplayer init failed: {}", e);
                 });
    player.print_info();
    
    wait_for_line("Press return to play");
    let _ = player.play();
    
    loop {
        println!("Position: {}", player.position());
        sleep_ms(100);
    }
}