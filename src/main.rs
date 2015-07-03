use std::path::Path;

extern crate demonplayer;

use demonplayer::Demonplayer;

/*fn wait_for_line(prompt: &str) {
    let mut reader = io::stdin();
    println!("{}", prompt);
    let mut line_buf = "".to_string();
    let _ = reader.read_line(&mut line_buf).unwrap();
}*/

fn main() {
    let player = Demonplayer::from_flac(&Path::new("test.flac"))
                     .unwrap_or_else(|e| {
                         panic!("demonplayer init failed: {:?}", e);
                     });

    println!("Sample rate: {}", player.sample_rate());
    println!("Bit depth: {}", player.bit_depth());
    println!("Channels: {}", player.channels());
    println!("Samples: {}", player.n_samples());
    println!("Duration: {} s", player.duration());

    /*player.print_info();
    
    wait_for_line("Press return to play");
    let _ = player.play();
    
    loop {
        println!("Position: {}", player.position());
        sleep_ms(100);
    }*/
}