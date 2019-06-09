extern crate time;

use std::f64::consts::PI;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

const RAD: f64 = 0.18;
const RAD2: f64 = RAD * RAD;
const W2: f64 = 0.3 * 0.3;
const DR: f64 = 0.002;
const DZ: f64 = 0.04;
const LAMBDA: f64 = 0.0106;
const AREA: f64 = PI * RAD2;
const Z1: f64 = PI * W2 / LAMBDA;
const Z12: f64 = Z1 * Z1;
const EXPR: f64 = 2 as f64 * PI * DR;
const INCR: usize = 8001;

#[derive(Debug, Default)]
struct Gaussian {
    input_power: u32,
    saturation_intensity: u32,
    output_power: f64,
}

#[derive(Debug)]
struct Laser {
    small_signal_gain: f32,
    discharge_pressure: u32,
    output_file: String,
    carbon_dioxide: String,
}

fn main() {
    let start = Instant::now();
    calculate();
    let elapsed = start.elapsed();
    println!(
        "The time was {}.{} seconds",
        elapsed.as_secs(),
        elapsed.subsec_nanos()
    );
}

fn calculate() {
    let pins_file = get_file_as_string("pin.dat");
    let inputs = get_input_powers(&pins_file);
    let inputs_rc = Arc::new(inputs);

    let laser_file = get_file_as_string("laser.dat");
    let lasers = get_laser_data(&laser_file);

    let mut handles = Vec::new();
    for laser in lasers {
        let input_ref = Arc::clone(&inputs_rc);
        let handle = thread::spawn(move || {
            process(&input_ref, &laser);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }
}

fn get_file_as_string(name: &str) -> String {
    let path = Path::new(name);
    let mut file = File::open(&path).expect(&format!("Can't open file {}", name));
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect(&format!("Cannot parse {} into a string", name));
    data
}

fn get_input_powers(data: &str) -> Vec<u32> {
    data.lines()
        .map(str::trim)
        .filter_map(|s| u32::from_str(s).ok())
        .collect()
}

fn get_laser_data(data: &str) -> Vec<Laser> {
    data.lines()
        .map(str::trim)
        .map(|s| Laser::from(s))
        .collect()
}

impl<'a> From<&'a str> for Laser {
    fn from(s: &str) -> Laser {
        let mut split = s.split_whitespace();
        Laser {
            output_file: split.next().unwrap().to_string(),
            small_signal_gain: split.next().unwrap().parse().unwrap(),
            discharge_pressure: split.next().unwrap().parse().unwrap(),
            carbon_dioxide: split.next().unwrap().to_string(),
        }
    }
}

fn process(input_powers: &Vec<u32>, laser: &Laser) {
    let path = Path::new(&laser.output_file);
    let mut fd = File::create(&path).unwrap();
    let header: String = format!(
        "Start date: {}\n\n\
         Gaussian Beam\n\n\
         Pressure in Main Discharge = {}kPa\n\
         Small-signal Gain = {:4.1}\n\
         CO2 via {}\n\n\
         Pin\t\tPout\t\tSat. Int\tln(Pout/Pin)\tPout-Pin\n\
         (watts)\t\t(watts)\t\t(watts/cm2)\t\t\t(watts)\n",
        time::strftime("%c", &time::now()).unwrap(),
        laser.discharge_pressure,
        laser.small_signal_gain,
        laser.carbon_dioxide
    );
    fd.write_all(header.as_bytes()).unwrap();

    for input in input_powers.iter() {
        let mut gaussian_data: [Gaussian; 16] = Default::default();
        gaussian_calculation(*input, laser.small_signal_gain, &mut gaussian_data);
        for gaussian in gaussian_data.iter() {
            let ln = (gaussian.output_power / gaussian.input_power as f64).ln();
            let pop = gaussian.output_power - gaussian.input_power as f64;
            let line: String = format!(
                "{}\t\t{:7.3}\t\t{}\t\t{:5.3}\t\t{:7.3}\n",
                gaussian.input_power, gaussian.output_power, gaussian.saturation_intensity, ln, pop
            );
            fd.write_all(line.as_bytes()).unwrap();
        }
    }

    let footer: String = format!(
        "End date: {}\n",
        time::strftime("%c", &time::now()).unwrap()
    );
    fd.write_all(footer.as_bytes()).unwrap();
}

fn gaussian_calculation(input_power: u32, small_signal_gain: f32, gaussian_data: &mut [Gaussian]) {
    let mut expr1: [f64; INCR] = [0 as f64; INCR];

    for i in 0..INCR {
        let z_inc = (i as f64 - (INCR as i32 / 2) as f64) / 25 as f64;
        let num = z_inc * 2 as f64 * DZ as f64;
        let dem = Z12 + z_inc.powi(2);
        expr1[i] = num as f64 / dem;
    }

    let input_intensity = ((2 * input_power) as f64) / AREA;
    let expr2 = small_signal_gain as f64 / 32000 as f64 * DZ;

    let mut i: usize = 0;
    let mut saturation_intensity = 10000;
    while saturation_intensity <= 25000 {
        let expr3 = saturation_intensity as f64 * expr2;
        let mut output_power: f64 = 0.0;
        let mut r = 0.0 as f32;
        while r <= 0.5 {
            let mut output_intensity =
                input_intensity * (-2 as f64 * r.powi(2) as f64 / RAD2).exp();
            for j in 0..INCR {
                output_intensity *=
                    1 as f64 + expr3 / (saturation_intensity as f64 + output_intensity) - expr1[j];
            }
            output_power += output_intensity * EXPR * r as f64;
            r += DR as f32;
        }
        gaussian_data[i].input_power = input_power;
        gaussian_data[i].saturation_intensity = saturation_intensity;
        gaussian_data[i].output_power = output_power;
        i += 1;

        saturation_intensity += 1000;
    }
}
