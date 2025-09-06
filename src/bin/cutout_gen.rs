//! G-Code generator for cutting out watch dials
//!
use core::f64;
use gcode::{
    g2_circle, g2_helix, gcode_comment, patterns, preamble, trailer, trimmed_g1_path, xy, xyf, xyr,
    xyzrf, PosRadiusAndFeed,
};
use std::fs::OpenOptions;
use std::io::{BufWriter, Result, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "cube_gen",
    about = "Generates a kind of guilloche-inspired pattern of QBert cubes"
)]
struct Opt {
    #[structopt(long, default_value = "16")]
    /// Outer radius
    outer_rad: f64,

    #[structopt(long, default_value = "0.4")]
    /// Cut depth
    depth: f64,

    /// Tool RPM
    #[structopt(long, default_value = "8000")]
    rpm: f64,

    /// Feed rate, in mm/min
    #[structopt(long, default_value = "300")]
    feed: f64,

    /// Feed rate, in mm/min
    #[structopt(long, default_value = "6.35")]
    tool_dia: f64,

    /// Name for the job
    #[structopt(short, long)]
    name: Option<String>,

    /// Tool number for the cut
    #[structopt(long, default_value = "15")]
    tool: u32,

    /// Output file for the resulting G code
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(long)]
    coolant: bool,
}

fn cutout(opt: &Opt, file: &mut dyn Write) -> Result<()> {
    let comp_rad = opt.outer_rad + opt.tool_dia / 2.0;
    // Feed down to near cutting depth

    g2_helix(
        file,
        xyzrf(0.0, 0.0, -opt.depth, comp_rad, opt.feed),
        1.0,
        0.1,
    )?;
    Ok(())
}

fn help_text(opt: &Opt) {
    println!(
        "Before cut:
        - Create stock with diameter at least {}mm
        - Set home to center of stock, at the top",
        opt.outer_rad * 2.0
    )
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    help_text(&opt);
    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&opt.output)?,
    );

    preamble(
        &opt.name,
        opt.tool,
        &format!("T{} D={} end mill", opt.tool, opt.tool_dia),
        opt.rpm,
        opt.coolant,
        &mut file,
    )?;
    cutout(&opt, &mut file)?;

    trailer(&mut file)?;

    file.flush()
}
