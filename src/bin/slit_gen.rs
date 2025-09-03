//! Generate a tool path for a slitting saw.
//!
//! The generated path is a single move along +X, possibly with cuts at multiple heights to make slits wider than the saw. This technique is controversial, but in my experience
//!  works well in brass, aluminium, and steel as long as the feed isn't too aggressive.
//!
//! The default speeds and feeds here work well on my Tormach 440, HSS saw, and into steel. "By the book" this is too much speed and too little feed, but the Tormach struggles
//!   with torque at the bottom of its RPM range, and so this approach is needed.
use gcode::{g0, g1, gcode_comment, preamble, trailer, x, xf, xyz, xyzf};
use std::f64::consts::PI;
use std::fs::OpenOptions;
use std::io::{BufWriter, Result, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "slit_gen", about = "Generates tool path for a slitting saw")]
struct Opt {
    /// Tool surface speed (in meters/minute)
    #[structopt(long, default_value = "120")]
    speed: f64,

    /// Feed rate per tooth, in mm/tooth
    #[structopt(long, default_value = "0.001")]
    feed_per_tooth: f64,

    /// Tool teeth
    #[structopt(long, default_value = "30")]
    tool_teeth: usize,

    /// Name for the job
    #[structopt(short, long)]
    name: Option<String>,

    /// Tool number for the cut
    #[structopt(long, default_value = "18")]
    tool: u32,

    /// Tool diameter, in mm
    #[structopt(long, default_value = "76.2")]
    tool_dia: f64,

    /// Tool thickness, in mm
    #[structopt(long, default_value = "1.55")]
    tool_thick: f64,

    /// Height of the cut, along -Z in mm, for making multiple passes with the saw. Leave unset for a single cut.
    #[structopt(long)]
    height: Option<f64>,

    /// Depth of the cut, along the X axis, mm
    #[structopt(long)]
    depth: f64,

    /// Output file for the resulting G code
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(long)]
    coolant: bool,
}

fn help_text() {
    println!(
        "Before cut:\n
            Align top of blade with top of cut.
            Set x, y, and z home along -X from cut"
    )
}

fn make_cut_pass(opt: &Opt, file: &mut dyn Write, z: f64, rpm: f64) -> Result<()> {
    let feed = opt.feed_per_tooth * rpm * opt.tool_teeth as f64;
    let z_clear = 4.0;

    assert!(z <= 0.0);

    gcode_comment(file, &format!("Making pass at z={z}"))?;
    // Rapid to our home
    g0(file, xyz(0.0, 0.0, z + z_clear))?;
    // Feed in slowly along Z, to give us an opportunity to panic
    g1(file, xyzf(0.0, 0.0, z, feed))?;
    // Feed in along the X axis
    g1(file, xf(opt.depth, feed))?;
    // Feed out along the X axis a little bit at the feed rate
    g1(file, xf(opt.depth - 1.0, feed))?;
    // Now rapid back to where we started
    g0(file, x(0.0))?;

    Ok(())
}

fn make_cut(opt: &Opt, file: &mut dyn Write, rpm: f64) -> Result<()> {
    let height = opt.height.unwrap_or(0.0);
    // First pass at the top height
    make_cut_pass(opt, file, 0.0, rpm)?;

    let bottom = height - opt.tool_thick;

    if height > opt.tool_thick {
        // Second pass at the bottom height
        make_cut_pass(opt, file, -bottom, rpm)?;
    }

    if height > opt.tool_thick * 2.0 {
        // Then make passes between the two until all the material has been cut away
        let start = opt.tool_thick;
        assert!(bottom > start);
        let passes = ((bottom - start) / opt.tool_thick).ceil();
        println!("end {bottom} start {start} passes {passes}");

        for i in 0..(passes as usize) {
            let z = start + i as f64 * (bottom - start) / passes;
            assert!(z < bottom);
            make_cut_pass(opt, file, -z, rpm)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    help_text();
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
        &format!(
            "T{} {}mm dia {}mm thick {} tooth slitting saw",
            opt.tool, opt.tool_dia, opt.tool_thick, opt.tool_teeth
        ),
        rpm,
        opt.coolant,
        &mut file,
    )?;
    make_cut(&opt, &mut file, rpm)?;
    trailer(&mut file)?;

    file.flush()
}
