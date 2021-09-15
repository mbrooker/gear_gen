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
    #[structopt(short, long, default_value="50")]
    cutter_dia: f64,

    /// Cutter RPM
    #[structopt(short, long, default_value="500")]
    rpm: f64,

    /// Feed rate, in mm/min
    #[structopt(short, long, default_value="50")]
    feed: f64,

    /// Name for the job
    #[structopt(short, long)]
    name: Option<String>,

    /// Tool number for the cut
    #[structopt(short, long, default_value="1")]
    tool: u32,

    /// Output file for the resulting G code
    #[structopt(parse(from_os_str))]
    output: PathBuf,
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

        G21 (Metric)

        G30 (Go Home Before Starting)
    ";
    write!(file, "{}\n\n", preamble_str)?;
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

    Ok(())
}
