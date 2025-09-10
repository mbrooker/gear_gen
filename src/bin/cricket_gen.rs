use anyhow::Result;
use core::f64;
use gcode::fonts::Font;
use gcode::{g0, g1, gcode_comment, preamble, trailer, xyz, xyzf, zf};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "cricket", about = "Makes cricket dice")]
struct Opt {
    #[structopt(long, default_value = "19")]
    /// Outer radius
    outer_rad: f64,

    #[structopt(long, default_value = "1")]
    /// Inner radius
    inner_rad: f64,

    #[structopt(long, default_value = "0.5")]
    /// Width of each turn in the spiral
    pass_width: f64,

    #[structopt(long, default_value = "0.1")]
    /// Depth for engraving
    depth: f64,

    #[structopt(long, default_value = "0.2")]
    /// Max stepdown per pass
    max_stepdown: f64,

    #[structopt(long, default_value = "5")]
    /// Number of 'rays' coming out from the center
    rays: usize,

    #[structopt(long, default_value = "0")]
    /// Radius 'wobble' in mm
    radial_wobble: f64,

    /// Tool RPM
    #[structopt(long, default_value = "8000")]
    rpm: f64,

    /// Number of steps to take around the circle
    #[structopt(long, default_value = "360")]
    steps_per_turn: usize,

    /// Feed rate, in mm/min
    #[structopt(long, default_value = "300")]
    feed: f64,

    /// Name for the job
    #[structopt(short, long)]
    name: Option<String>,

    /// Tool number for the cut
    #[structopt(long, default_value = "17")]
    tool: u32,

    /// Output file for the resulting G code
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(long)]
    coolant: bool,
}

fn engrave_text(file: &mut dyn Write, opt: &Opt, font: &Font, s: &str) -> Result<()> {
    let safe_z = 1.0;
    font.string_to_gcode(file, s, &xyzf(0.0, 0.0, opt.depth, opt.feed), safe_z, 5.0)?;
    Ok(())
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(&opt.output)?,
    );

    preamble(
        &opt.name,
        opt.tool,
        &format!("T{} D={} engraver", opt.tool, opt.pass_width),
        opt.rpm,
        opt.coolant,
        &mut file,
    )?;
    let font = Font::new_from_svg(&PathBuf::from_str("EMSReadability.svg")?)?;
    engrave_text(&mut file, &opt, &font, "This is a test 0123456789")?;
    trailer(&mut file)?;

    file.flush()?;
    Ok(())
}
