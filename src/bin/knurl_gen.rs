///! G-Code generator for cutting knurling tools on a rotational axis
///! This is designed for cutting with engraving or chamfering tools: a mill with a sharp end.
///! The included angle (and depth) of the teeth depends on the included angle of the tool.
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

/// Calculate the feed rate we need to tell the machine to get a real surface feed rate of `target_feed`, in units of
/// 1/minutes (for G93 inverse feed rate mode)
/// LinuxCNC says this about the way feed rate is interpreted during simultaneous multi-axis:
///   "If any of XYZ are moving, F is in units per minute in the XYZ cartesian system, and all
///    other axes (ABCUVW) move so as to start and stop in coordinated fashion."
/// So we have to correct the feed rate we get from the machine to get the right actual feed at the tip of the tool. Doing
///  that in a way that machines agree on seems hard, so instead we use G93 mode and let the machine figure out the
///  XYZ and ABC feed rates.
fn calc_feed_g93(opt: &Opt) -> f64 {
    // How much we adjust the feed to compensate for simultaneous rotary motion
    let cutting_path_length = opt.len / opt.spiral_angle.to_radians().cos(); 
    cutting_path_length / opt.feed
}

fn cut_tooth(opt: &Opt, file: &mut dyn Write, teeth: usize, a_start: f64) -> Result<()> {
    // Cutting a knurl consists of making a number of passes at different depths until we arrive at the final depth
    
    // How far away we want to keep the tool from the work when not cutting
    let clearance = 4.0;

    // We're always cutting along the `y` axis at y=0
    let tool_y = 0.0;
    let stock_top_z = opt.dia / 2.0;

    let actual_tooth_width = (PI * opt.dia) / (teeth as f64);
    let tooth_depth = (actual_tooth_width / 2.0) / (opt.tool_inc_angle.to_radians().tan());
    let passes = (tooth_depth / opt.max_stepdown).ceil() as usize;
    let actual_stepdown = tooth_depth / passes as f64;

    // Calculate the ending angle for the spiral, in degrees
    let a_end = a_start + 360.0 * opt.len * opt.spiral_angle.to_radians().tan() / (PI * opt.dia);


    let cutting_feed = calc_feed_g93(opt);

    for i in 0..passes {
        let z = stock_top_z - actual_stepdown * i as f64;
        g0(file, xyza(clearance, tool_y, stock_top_z + clearance, a_start))?;
        // Plunge the tool to z depth. Shouldn't be cutting yet, but we're being a bit careful
        g1(file, zf(z, opt.feed))?;
        // Feed in along the x axis until the tool is about to make contact
        g1(file, xf(0.1, opt.feed))?;

        // Simultaneously move in X and A, cutting the actual path
        inv_feed_g93(file)?;
        g1(file, xaf(-opt.len, a_end, cutting_feed))?;
        standard_feed_g94(file)?;

        // Move out of the work in Z to the clearance height
        g1(file, zf(stock_top_z + clearance, opt.feed))?;
    }

    Ok(())

}

fn cut_knurls(opt: &Opt, file: &mut dyn Write) -> Result<()> {
    let circumference = PI * opt.dia;
    let teeth = (circumference / opt.pitch).floor() as usize;
    let a_step = 360.0 / teeth as f64;



    for i in 0..teeth {
        gcode_comment(file, &format!("Tooth {} of {}", i + 1, teeth))?;
        cut_tooth(opt, file, teeth, a_step * i as f64)?;
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