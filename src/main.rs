use std::path::PathBuf;
use structopt::StructOpt;
use std::fs::{OpenOptions, File};
use std::io::{Write, Result};

#[derive(Debug, StructOpt)]
#[structopt(name = "gear_gen", about = "A simple spur gear generator")]
struct Opt {
    /// Gear module, must match cutter module
    #[structopt(short, long, default_value="1")]
    module: f64,

    /// Number of gear teeth
    #[structopt(short, long)]
    teeth: u32,

    /// Diameter of cutter, in mm
    #[structopt(long, default_value="50")]
    cutter_dia: f64,

    /// Cutter RPM
    #[structopt(long, default_value="500")]
    rpm: f64,

    /// Feed rate, in mm/min
    #[structopt(long, default_value="50")]
    feed: f64,

    /// Name for the job
    #[structopt(short, long)]
    name: Option<String>,

    /// Tool number for the cut
    #[structopt(long, default_value="1")]
    tool: u32,

    /// Width of the gear to cut
    #[structopt(short, long)]
    width: f64,

    /// Max depth to cut, in mm
    #[structopt(long, default_value="0.5")]
    max_depth: f64,

    /// Output file for the resulting G code
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(long)]
    coolant: bool,
}

fn gcode_comment(file: &mut File, s: &str) -> Result<()> {
    write!(file, "({})\n", s)
}

fn preamble(opt: &Opt, file: &mut File) -> Result<()> {
    // Print out the name as a comment on the first line, if set
    if let Some(name) = &opt.name {
       gcode_comment(file, &name)?;
    }
    // Comment with tool information
    gcode_comment(file, &format!("T{} D={} - gear mill", opt.tool, opt.cutter_dia))?;
    
    // Preamble to set the machine into a reasonable mode
    let preamble_str = "
G90 (Absolute)
G54 (G54 Datum)
G17 (X-Y Plane)
G40 (No cutter compensation)
G80 (No cycles)
G94 (Feed per minute)
G91.1 (Arc absolute mode)
G49 (No tool length compensation)
M9 (Coolant off)

G21 (Metric)

G30 (Go Home Before Starting)
    ";
    write!(file, "{}\n\n", preamble_str)?;
    // Print the tool mode preamble, choosing the tool, 
    // enabling length compensation,
    // and executing the tool change cycle
    write!(file, "T{} G43 H{} M6\n", opt.tool, opt.tool)?;

    // Print the Speed preamble, and turn on the spindle
    write!(file, "S{} M3\n", opt.rpm)?;

    // If chosen, start coolant flowing
    if opt.coolant {
        write!(file, "M8\n")?;
    }

    Ok(())
}

fn trailer(_opt: &Opt, file: &mut File) -> Result<()> {
    write!(file, "M9 (Coolant off)\n")?;
    write!(file, "M5 (Spindle off)\n")?;
    write!(file, "M30\n")?;

    Ok(())
}

fn pass_at_depth(opt: &Opt, file: &mut File, depth: f64) -> Result<()> {
    let y_pos = (opt.teeth as f64 + 2.0)*opt.module + opt.cutter_dia/2.0 - depth;
    gcode_comment(file, &format!("Pass at depth {}", depth))?;
    // Go to our starting point, to the right of the stock
    write!(file, "G0 X{} Y{}\n", opt.cutter_dia/2.0, y_pos)?;
    write!(file, "G0 Z0.\n")?;

    // Feed into the stock, cutting as we go
    write!(file, "G1 X{} F{}\n", -opt.width, opt.feed)?;

    // Feed out of the stock, moving in Y
    // TODO: This feed-out should probably be radiused, to avoid backlash issues
    write!(file, "G1 Y{} F{}\n", y_pos + 10.0, opt.feed)?;

    // Go back to where we started, in two moves, first X then Y to make sure we have enough clearance
    write!(file, "G0 X{}\n", opt.cutter_dia/2.0)?;
    write!(file, "G0 Y{}\n", y_pos)?;

    Ok(())
}

fn cut_tooth(opt: &Opt, file: &mut File, angle: f64) -> Result<()> {
    // First, turn the rotary axis to the right angle, rapid
    write!(file, "G0 A{}\n", angle)?;
    
    // Total depth varies from source to source.
    // Here, I'm using the formula from the Machinery's Handbook, 31st Edition, "Module System Gear Design"
    let mut depth = 2.157 * opt.module;
    // Take passes until we've consumed the whole depth.
    // TODO: Avoid making the last pass two small, by implementing a minimum pass depth too.
    while depth > 0.0 {
        pass_at_depth(opt, file, depth.min(opt.max_depth))?;
        depth -= opt.max_depth;
    }

    // Go home between teeth
    write!(file, "G30\n")?;

    Ok(())
}

fn cut_teeth(opt: &Opt, file: &mut File) -> Result<()> {
    let tooth_angle = 360.0 / opt.teeth as f64;

    for i in 0..opt.teeth {
        cut_tooth(opt, file, i as f64 * tooth_angle)?;
    }

    Ok(())
}

fn help_text(opt: &Opt) {
    println!("Before cut:
        - Create stock with OD {}mm
        - Set home to center of right face of stock",
    (opt.teeth+2) as f64*opt.module)
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    help_text(&opt);
    let mut file = OpenOptions::new().write(true)
        .create_new(true)
        .open(&opt.output)?;

    preamble(&opt, &mut file)?;
    cut_teeth(&opt, &mut file)?;
    trailer(&opt, &mut file)?;

    Ok(())
}
