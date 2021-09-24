///! G-Code generator for cutting fluted mill tools

///! For an example of where I use this, see http://www.helicron.net/workshop/gearcutting/gear_cutter/
///! We don't do the actual tooth cutting here (yet), that still needs to be done on a lathe. This just turns the round
///! hobber into a tool with sharp teeth and back relief behind the teeth.
use gcode::{g0, g1, gcode_comment, preamble, trailer, xyza, zaf, zf};
use std::fs::{File, OpenOptions};
use std::io::{Result, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "hobber",
    about = "Generates tool paths to cut teeth into hobbers"
)]
struct Opt {
    /// Number of flutes in the cutter
    #[structopt(long)]
    flutes: u32,

    /// Max depth of each flute, in mm
    #[structopt(long)]
    depth: f64,

    /// Unrelieved angle, i.e. the angle of each tooth in A before the back relief starts, in degrees
    #[structopt(long, default_value = "10")]
    unrelieved_angle: f64,

    /// Length of the cutter we're creating, in mm
    #[structopt(long, default_value = "20")]
    len: f64,

    /// Diameter of cutter we're creating, in mm
    #[structopt(long)]
    dia: f64,

    /// Tool RPM
    #[structopt(long, default_value = "650")]
    rpm: f64,

    /// Feed rate, in mm/min
    #[structopt(long, default_value = "60")]
    feed: f64,

    /// Name for the job
    #[structopt(short, long)]
    name: Option<String>,

    /// Tool number for the cut
    #[structopt(long, default_value = "1")]
    tool: u32,

    /// Diameter of tool, in mm
    #[structopt(long, default_value = "3")]
    tool_dia: f64,

    /// Max cutting stepdown, per pass, in mm
    #[structopt(long, default_value = "3")]
    max_stepdown: f64,

    /// Tool stepover, as a ratio of tool width (i.e. 0.5 steps over by half the tool diameter).
    #[structopt(long, default_value = "0.25")]
    max_stepover: f64,

    /// Spiral angle. 0 for straight flutes, higher values for spiral flutes. In degrees
    #[structopt(long, default_value = "25")]
    spiral_angle: f64,

    /// Output file for the resulting G code
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(long)]
    coolant: bool,
}

fn pass_at_depth(
    opt: &Opt,
    file: &mut File,
    x_pos: f64,
    max_depth: f64,
    a_start: f64,
) -> Result<()> {
    // Clearance (in mm) away from the stock where we move at feed rate
    let clearance = 4.0;

    // All ops happen along the "top" of the stock, minus some Z depth, moving in A and -Z simultaneously
    let y_pos = 0.0;

    let z_start = opt.dia / 2.0;
    let z_end = z_start - max_depth;

    let a_end = a_start + 360.0 / opt.flutes as f64 - opt.unrelieved_angle;

    gcode_comment(file, &format!("Pass at depth {}", max_depth))?;
    // Rapid to some distance above the start of the work
    g0(file, xyza(x_pos, y_pos, z_start + clearance, a_start))?;
    // Feed in to the starting Z at feed rate (this shouldn't plunge the tool, but we're just being cautious by not making this rapid)
    g1(file, zf(z_start, opt.feed))?;
    // Now simultaneously feed in the Z and A axes
    g1(file, zaf(z_end, a_end, opt.feed))?;
    // Then feed out back to the Z clearance point
    g1(file, zf(z_start, opt.feed))?;

    Ok(())
}

fn cut_flute(opt: &Opt, file: &mut File, angle: f64) -> Result<()> {
    let mut x = 0.0;
    // Take passes until we've consumed the whole X distance
    while x > -opt.len {
        let angle_on_spiral = angle + 360.0 * x * opt.spiral_angle.to_radians().tan() / (std::f64::consts::PI * opt.dia);
        let mut depth = 0.0;
        // Take passes until we've consumed the whole target depth
        while depth < opt.depth {
            depth = (depth + opt.max_stepdown).clamp(0.0, opt.depth);
            pass_at_depth(opt, file, x, depth, angle_on_spiral)?;
        }
        // Move up the x axis by our stepover value
        x -= opt.tool_dia * opt.max_stepover;
    }

    // Go home between teeth
    write!(file, "G30\n\n")?;

    Ok(())
}

fn cut_flutes(opt: &Opt, file: &mut File) -> Result<()> {
    let flute_angle = 360.0 / opt.flutes as f64;

    for i in 0..opt.flutes {
        gcode_comment(file, &format!("Flute {} of {}", i + 1, opt.flutes))?;
        cut_flute(opt, file, i as f64 * flute_angle)?;
    }

    Ok(())
}

fn help_text(opt: &Opt) {
    println!(
        "Before cut:
        - Create stock with OD {}mm
        - Set home to center of right face of stock",
        opt.dia
    )
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    help_text(&opt);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&opt.output)?;

    preamble(
        &opt.name,
        opt.tool,
        opt.tool_dia,
        opt.rpm,
        opt.coolant,
        &mut file,
    )?;
    cut_flutes(&opt, &mut file)?;
    trailer(&mut file)?;

    Ok(())
}
