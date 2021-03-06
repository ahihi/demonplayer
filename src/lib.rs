extern crate claxon;
extern crate portaudio;

use claxon::frame::FrameReader;
use portaudio::pa;
use std::convert::From;
use std::error::Error;
use std::fs::File;
use std::io;
use std::mem;
use std::path::Path;
use std::result::Result;
use std::sync::Arc;
use std::sync::RwLock;

const SAMPLE_FORMAT: pa::SampleFormat = pa::SampleFormat::Int32;
pub type DSample = i32;
pub type DStream = pa::Stream<DSample, DSample>;
pub type DCallback = pa::StreamCallbackFn<DSample, DSample>;

#[derive(Debug)]
pub enum DError {
    Io(io::Error),
    Claxon(claxon::error::Error),
    Pa(pa::error::Error)
}

impl From<io::Error> for DError {
    fn from(e: io::Error) -> DError {
        DError::Io(e)
    }
}

impl From<claxon::error::Error> for DError {
    fn from(e: claxon::error::Error) -> DError {
        DError::Claxon(e)
    }
}

impl From<pa::error::Error> for DError {
    fn from(e: pa::error::Error) -> DError {
        DError::Pa(e)
    }
}

pub type DResult<T> = Result<T, DError>;

const FRAMES_PER_BUFFER: u32 = 512;

pub enum PlayState {
    Paused,
    Playing
}

struct PlayerState {
    stream: DStream,
    start_time: pa::Time,
    index: usize,
    play_state: PlayState
}

impl PlayerState {
    fn new(stream: DStream) -> Self {
        PlayerState {
            stream: stream,
            start_time: 0.0,
            index: 0,
            play_state: PlayState::Paused
        }
    }
}

pub struct Demonplayer {
    //output_name: String,
    flac_info: claxon::metadata::StreamInfo,
    n_samples: usize,
    out_stream_params: pa::StreamParameters,
    state: Arc<RwLock<PlayerState>>
}

impl Demonplayer {
    pub fn from_flac(path: &Path) -> DResult<Demonplayer> {
        let (info, n_samples, buffer) = try!(Self::read_flac(path));

        println!("Init audio");
        let (_output_name, out_stream_params, stream)
            = try!(Self::init_audio(info.sample_rate as f64));

        println!("Create player");
        let state = Arc::new(RwLock::new(PlayerState::new(stream)));
        let cb_state = state.clone();

        // Define callback
        let callback = Box::new(move |
            _input: &[DSample],
            output: &mut[DSample],
            frames: u32,
            _time_info: &pa::StreamCallbackTimeInfo,
            _flags: pa::StreamCallbackFlags
        | -> pa::StreamCallbackResult {
            assert!(frames == FRAMES_PER_BUFFER);
            
            let mut state = cb_state.write().unwrap();
            for output_sample in output.iter_mut() {
                let (sample, index_inc) = match state.play_state {
                    PlayState::Paused => (0, 0),
                    PlayState::Playing => if state.index < buffer.len() {
                        (buffer[state.index], 1)
                    } else {
                        (0, 0)
                    }
                };
                    
                *output_sample = sample;
                state.index += index_inc;
            }

            if state.index < buffer.len() {
                pa::StreamCallbackResult::Continue
            } else {
                pa::StreamCallbackResult::Complete
            }
        });

        // Register callback
        println!("Register callback");
        {
            let mut s = (&state).write().unwrap();
            try!(s.stream.open(
                None,
                Some(&out_stream_params),
                info.sample_rate as f64,
                FRAMES_PER_BUFFER,
                pa::StreamFlags::empty(),
                Some(callback)                
            ));
        }
        
        println!("Done");

        let player = Demonplayer {
            flac_info: info,
            n_samples: n_samples,
            out_stream_params: out_stream_params,
            state: state
        };

        Ok(player)
    }

    fn read_flac(path: &Path) -> DResult<(claxon::metadata::StreamInfo, usize, Vec<DSample>)> {
        // Open the flac stream
        println!("Open stream");
        let file = try!(File::open(path));
        let mut reader = io::BufReader::new(file);
        let mut stream = try!(claxon::FlacStream::new(&mut reader));

        // Get stream info
        println!("Get stream info");
        let info = *stream.streaminfo();
        let n_samples = info.n_samples
                        .unwrap_or_else(|| {
                            panic!("n_samples = None")
                        }) as usize;

        // Read the entire stream into a buffer
        println!("Make buffer");
        let buffer_size = info.n_channels as usize * n_samples;
        let mut buffer = Vec::<i32>::with_capacity(buffer_size);
        unsafe { buffer.set_len(buffer_size) };

        println!("Fill buffer");
        let sample_shift = 8*mem::size_of::<DSample>() - info.bits_per_sample as usize;
        println!("sample_shift = {}", sample_shift);
        let mut frame_reader: FrameReader<i32> = stream.blocks();
        let mut sample_offset = 0usize;
        while let Ok(block) = frame_reader.read_next() {
            let channels = block.channels();
            for i_ch in 0 .. channels {
                let ch = block.channel(i_ch);
                for (i_sample, sample) in ch.iter().enumerate() {
                    let i_buffer = (channels as usize)*(sample_offset + i_sample) + (i_ch as usize);
                    buffer[i_buffer] = (*sample) << sample_shift;
                }
            }
            sample_offset += block.len() as usize;
        }

        Ok((info, n_samples, buffer))
    }

    fn init_audio(sample_rate: f64) -> DResult<(String, pa::StreamParameters, DStream)> {
        try!(pa::initialize());

        let default_output = pa::device::get_default_output();
        let output_info = try!(pa::device::get_info(default_output));

        let out_stream_params = pa::StreamParameters {
            device:             default_output,
            channel_count:      2,
            sample_format:      SAMPLE_FORMAT,
            suggested_latency:  output_info.default_low_output_latency,
        };
        try!(pa::is_format_supported(None, Some(&out_stream_params), sample_rate));

        let stream: DStream = pa::Stream::new();

        Ok((output_info.name, out_stream_params, stream))
    }

    pub fn sample_rate(&self) -> u32 {
        self.flac_info.sample_rate
    }

    pub fn bit_depth(&self) -> u8 {
        self.flac_info.bits_per_sample
    }

    pub fn channels(&self) -> u8 {
        self.flac_info.n_channels
    }

    pub fn n_samples(&self) -> usize {
        self.n_samples
    }

    pub fn duration(&self) -> f32 {
        (self.n_samples() as f32) / (self.sample_rate() as f32)
    }

    pub fn play(&self) -> DResult<()> {
        let mut state = self.state.write().unwrap();
        state.play_state = PlayState::Playing;
        state.start_time = state.stream.get_stream_time();
        try!(state.stream.start());
        
        Ok(())        
    }
        
    pub fn position(&self) -> Option<pa::Time> {
        let state = self.state.read().unwrap();
        if let Ok(true) = state.stream.is_active() {
            Some(state.stream.get_stream_time() - state.start_time)
        } else {
            None
        }
    }

    /*pub fn print_info(&self) {
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
    }*/
}

impl Drop for Demonplayer {
    fn drop(&mut self) {
        let mut state = self.state.write().unwrap();
        state.stream.close()
        .unwrap_or_else(|e| {
            println!("stream.close() failed: {}", e.description());
        });

        pa::terminate()
        .unwrap_or_else(|e| {
            println!("pa::terminate() failed: {}", e.description());
        });
    }
}
