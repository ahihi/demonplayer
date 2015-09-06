use std::io;
use std::path::Path;
use std::thread;

extern crate demonplayer;

use demonplayer::Demonplayer;

#[derive(Debug)]
enum Command {
    PlayPause
}

impl Command {
    fn from_str(text: &str) -> Option<Self> {
        match text {
            "p" => Some(Command::PlayPause),
            _   => None
        }
    }
}

fn read_command(input: &mut io::Stdin) -> Option<Command> {
    let mut line_buf = "".to_string();
    let _ = input.read_line(&mut line_buf).unwrap();
    
    Command::from_str(&line_buf.trim_right())
}

fn main() {
    let player = Demonplayer::from_flac(&Path::new("test.flac"))
                 .unwrap_or_else(|e| {
                     panic!("demonplayer init failed: {:?}", e);
                 });

    println!("");
    println!("Sample rate: {}", player.sample_rate());
    println!("Bit depth: {}", player.bit_depth());
    println!("Channels: {}", player.channels());
    println!("Samples: {}", player.n_samples());
    println!("Duration: {} s", player.duration());

    println!("");
    println!("Starting playback");
    let _ = player.play();
    
    let mut stdin = io::stdin();
    while let Some(pos) = player.position() {
        println!("Position: {}", pos);
        thread::sleep_ms(100);
        /*
        println!("");
        println!("Enter command (p = play/pause):");
        match read_command(&mut stdin) {
            Some(cmd)   => println!("{:?}", cmd),
            None        => println!("Unknown command"),
        }
        */
    }
}
