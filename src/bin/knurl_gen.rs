///! G-Code generator for cutting knurling tools on a rotational axis
///! This is designed for cutting with engraving or chamfering tools: a mill with a sharp end.
///! The included angle (and depth) of the teeth depends on the included angle of the tool.
use gcode::{
    g0, g1, gcode_comment, inv_feed_g93, preamble, standard_feed_g94, trailer, xyza, zaf, zf,
};
use std::f64::consts::PI;
use std::fs::OpenOptions;
use std::io::{BufWriter, Result, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "knurl_gen",
    about = "Generates tool paths to create knurling tools"
)]
struct Opt {
    /// Length (along A axis) of the knurler we're creating, in mm
    #[structopt(long, default_value = "10")]
    len: f64,

    /// Diameter of knurler we're creating, in mm
    #[structopt(long)]
    dia: f64,

    /// Tool RPM
    #[structopt(long, default_value = "4500")]
    rpm: f64,

    /// Feed rate, in mm/min
    #[structopt(long, default_value = "220")]
    feed: f64,

    /// Name for the job
    #[structopt(short, long)]
    name: Option<String>,

    /// Tool number for the cut
    #[structopt(long, default_value = "1")]
    tool: u32,

    /// Diameter of tool, in mm
    #[structopt(long, default_value = "3.175")]
    tool_dia: f64,

    /// Tool included angle, in degrees
    #[structopt(long, default_value = "60")]
    tool_inc_angle: f64,

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
        "Before cut:
        - Create stock with OD {}mm
        - Set home to center of right face of stock",
        opt.dia
    )
}

fn cut_knurl(opt: &Opt, file: &mut dyn Write) -> Result<()> {

    Ok(())

}

fn cut_knurls(opt: &Opt, file: &mut dyn Write) -> Result<()> {
    let circumference = PI * opt.dia;
    let teeth = (circumference / opt.pitch).floor() as usize;
    let actual_tooth_width = circumference / (teeth as f64);
    let tooth_depth = (actual_tooth_width / 2.0) / (opt.tool_inc_angle.tan());

    // How much we adjust the feed to compensate for simultaneous rotary motion
    let feed_adjustment = opt.spiral_angle.cos(); 

    for i in 0..teeth {
        gcode_comment(file, &format!("Tooth {} of {}", i + 1, teeth))?;
        cut_knurl(opt, file)?;
    }

    Ok(())
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

    preamble(
        &opt.name,
        opt.tool,
        opt.tool_dia,
        opt.rpm,
        opt.coolant,
        &mut file,
    )?;
    cut_knurls(&opt, &mut file)?;
    trailer(&mut file)?;

    file.flush()
}