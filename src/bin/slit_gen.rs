//! Generate a tool path for a slitting saw

use gcode::{
    g0, g1, gcode_comment, inv_feed_g93, preamble, standard_feed_g94, trailer, xyza, zf, xf, xaf,
};
use std::f64::consts::PI;
use std::fs::OpenOptions;
use std::io::{BufWriter, Result, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "slit_gen",
    about = "Generates tool path for a slitting saw"
)]
struct Opt {
    /// Depth (along X axis) of the slit we are creating
    #[structopt(long, default_value = "16")]
    len: f64,

    /// Tool surface speed (in meters/minute)
    // Feed and speed defaults for 1/4" carbide in annealed W1
    #[structopt(long, default_value = "40")]
    speed: f64,

    /// Feed rate, in mm/tooth
    #[structopt(long, default_value = "0.0254")]
    feed: f64,

    /// Tool teeth
    #[structopt(long, default_value = "30")]
    tool_teeth: usize,

    /// Name for the job
    #[structopt(short, long)]
    name: Option<String>,

    /// Tool number for the cut
    #[structopt(long, default_value = "1")]
    tool: u32,

    /// Tool diameter, in mm
    #[structopt(long, default_value = "76.2")]
    tool_dia: f64,

    /// Knurling tool pitch (in mm per tooth). Typical tools vary from 1.6 (15tpi) to 0.75 (33tpi). 
    #[structopt(long, default_value="1")]
    pitch: f64,

    /// Max cutting stepdown, per pass, in mm
    #[structopt(long, default_value = "0.25")]
    max_stepdown: f64,

    /// Spiral angle (degrees). 0 for straight-cut knurler, 45 for diamond.
    #[structopt(long, default_value = "45")]
    spiral_angle: f64,

    /// Output file for the resulting G code
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(long)]
    coolant: bool,
}

fn help_text(opt: &Opt) {
    println!(
        "Before cut:"
    )
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    help_text(&opt);
    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&opt.output)?,
    );

    // Calculate the RPM from the surface speed
    let rpm = opt.speed / (PI * (opt.tool_dia / 1000.0));

    preamble(
        &opt.name,
        opt.tool,
        &format!("T{} {}mm {} tooth slitting saw", opt.tool, opt.tool_dia, opt.tool_teeth),
        rpm,
        opt.coolant,
        &mut file,
    )?;
    
    trailer(&mut file)?;

    file.flush()
}