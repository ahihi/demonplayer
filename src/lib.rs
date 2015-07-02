extern crate portaudio;

use portaudio::pa;
use std::error::Error;
use std::f32::consts;
use std::result::Result;

pub type DSample = f32;
pub type DStream = pa::Stream<DSample, DSample>;
pub type DResult<T> = Result<T, pa::error::Error>;
const SAMPLE_FORMAT: pa::SampleFormat = pa::SampleFormat::Float32;
const SAMPLE_RATE: f64 = 44100.0;
const FRAMES_PER_BUFFER: u32 = 512;

pub struct Demonplayer {
    output_name: String,
    stream: DStream,
}

fn sine(freq: f32, i: u64) -> DSample {
    ((i as f32) / (SAMPLE_RATE as f32) * freq * 2.0 * consts::PI).sin()
}

fn sines(i: u64) -> DSample {
    let low = sine(220.0, i);
    let high = sine(340.0, i);
    let low_amp = 0.3 * sine(0.08, i);
    let high_amp = 0.2 * sine(0.1, i);
    
    low_amp*low + high_amp*high
}

impl Demonplayer {    
    pub fn new() -> DResult<Demonplayer> {
        try!(pa::initialize());
        
        let default_output = pa::device::get_default_output();
        let output_info = try!(pa::device::get_info(default_output));
                          
        let output_stream_params = pa::StreamParameters {
            device:             default_output,
            channel_count:      2,
            sample_format:      SAMPLE_FORMAT,
            suggested_latency:  output_info.default_low_output_latency,
        };
        try!(pa::is_format_supported(None, Some(&output_stream_params), SAMPLE_RATE));
        
        let mut stream: DStream = pa::Stream::new();
        
        // Once the countdown reaches 0 we'll close the stream.
        //let mut count_down = 3.0;

        // Keep track of the last `current_time` so we can calculate the delta time.
        //let mut maybe_last_time = None;
        
        let mut sample_i: u64 = 0;
        
        // Construct a custom callback function - in this case we're using a FnMut closure.
        let callback = Box::new(move |
            _input: &[f32],
            output: &mut[f32],
            frames: u32,
            _time_info: &pa::StreamCallbackTimeInfo,
            _flags: pa::StreamCallbackFlags,
        | -> pa::StreamCallbackResult {

            //let current_time = time_info.current_time;
            //let prev_time = maybe_last_time.unwrap_or(current_time);
            //let dt = current_time - prev_time;
            //count_down -= dt;
            //maybe_last_time = Some(current_time);

            assert!(frames == FRAMES_PER_BUFFER);
            //sender.send(count_down).ok();
            
            for output_sample in output.iter_mut() {
                *output_sample = sines(sample_i);
                sample_i += 1;
            }

            /*if count_down > 0.0 {
                pa::StreamCallbackResult::Continue
            } else {
                pa::StreamCallbackResult::Complete
            }*/
            pa::StreamCallbackResult::Continue
        });

        try!(stream.open(
            None,
            Some(&output_stream_params),
            SAMPLE_RATE,
            FRAMES_PER_BUFFER,
            pa::StreamFlags::empty(),
            Some(callback)
        ));
        
        Ok(Demonplayer {
            output_name:    output_info.name,
            stream:         stream,
        })
    }
    
    pub fn play(&mut self) -> DResult<()> {
        self.stream.start()
    }
    
    pub fn print_info(&self) {
        let api_name = pa::host::get_api_info(pa::host::get_default_api())
                       .unwrap_or_else(|| {
                           panic!("No info for default API");
                       })
                       .name;
        println!("Demonplayer API: {}", api_name);
        println!("Demonplayer output: {}", self.output_name);
    }
    
    pub fn api(&self) -> Option<String> {
        let default_host = pa::host::get_default_api();
        let api_info = pa::host::get_api_info(default_host);
        let api_name = match api_info {
            None       => None,
            Some(info) => Some(info.name),
        };
        api_name
    }
}

impl Drop for Demonplayer {
    fn drop(&mut self) {
        self.stream.close()
        .unwrap_or_else(|e| {
            println!("stream.close() failed: {}", e.description());
        });
        
        pa::terminate()
        .unwrap_or_else(|e| {
            println!("pa::terminate() failed: {}", e.description());
        });
    }
}
