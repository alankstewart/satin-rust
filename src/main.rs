use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufRead};

struct Laser {
    output_file: String,
    small_signal_gain: f64,
    discharge_pressure: u32,
    carbon_dioxide: String,
}

struct Gaussian {
    input_power: u32,
    saturation_intensity: u32,
    output_power: f64,
}

fn main() {
    println!("Hello, world!");

    let path = Path::new("/Users/alankstewart/projects/satin-rust/src/pin.dat");
    let reader = BufReader::new(File::open(&path).expect("File not found"));
    for line in reader.lines() {
        println!("{}", line.unwrap());
    }

}
