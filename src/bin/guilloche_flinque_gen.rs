use core::f64;
///! G-Code generator for a kind of wavy spiral guilloche
use gcode::{g0, g1, gcode_comment, preamble, trailer, xyz, xyzf, zf, z};
use std::fs::OpenOptions;
use std::io::{BufWriter, Result, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "guilloche",
    about = "Generates a kind of guilloche-inspired spiral pattern with varying depth"
)]
struct Opt {
    #[structopt(long, default_value = "19")]
    /// Outer radius
    outer_rad: f64,

    #[structopt(long, default_value = "1")]
    /// Inner radius
    inner_rad: f64,

    #[structopt(long, default_value = "0.75")]
    /// Gap between circles
    step_over: f64,

    #[structopt(long, default_value = "0.2")]
    /// Cut depth
    depth: f64,

    #[structopt(long, default_value = "17")]
    /// Number of 'rays' coming out from the center
    rays: usize,

    #[structopt(long, default_value = "1")]
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

fn generate_flinque(opt: &Opt, file: &mut dyn Write) -> Result<()> {
    let circles = (opt.outer_rad / opt.step_over).floor() as usize;
    let skip_circles = (opt.inner_rad / opt.step_over).ceil() as usize;

    let start_x = skip_circles as f64 * opt.step_over;

    // Rapid the starting position
    g0(file, xyz(start_x, 0., 10.0))?;
    g0(file, xyz(start_x, 0., 1.0))?;


    for circle in skip_circles..circles {
        gcode_comment(file, &format!("Circle: {}", circle))?;
        // Rapid over to the start position for the next circle
        g0(file, xyz(circle as f64 * opt.step_over + opt.radial_wobble / 2.0, 0., 1.0))?;
        // Then slowly plunge to depth
        g1(file, zf(-opt.depth, opt.feed))?;
        // Loop around, plus a little overlap
        for angle_step in 0..(opt.steps_per_turn + 5) {
            let angle = 2.0 * f64::consts::PI * angle_step as f64 / opt.steps_per_turn as f64;
            // In range [0, 1], where we are on the z cycle
            let wobble = opt.radial_wobble * (1.0 + (angle * opt.rays as f64).sin()) / 2.0;

            let radius = circle as f64 * opt.step_over + wobble;
            let x = radius * angle.cos();
            let y = radius * angle.sin();

            g1(file, xyzf(x, y, -opt.depth, opt.feed))?;
        }
        // Rapid out
        g0(file, z(1.0))?;
    }
    // Rapid out
    g0(file, z(10.0))?;
    Ok(())
}



fn help_text(opt: &Opt) {
    let circles = (opt.outer_rad / opt.step_over).floor() as usize;
    let total_distance = circles as f64 * f64::consts::PI * opt.outer_rad;

    println!(
        "Before cut:
        - Create stock with diameter at least {}mm
        - Set home to center of stock, at the top,
        - Travel distance {}mm,
        - Approx run time {} minutes",
        opt.outer_rad * 2.0,
        total_distance,
        total_distance / opt.feed,
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

    preamble(
        &opt.name,
        opt.tool,
        &format!("T{} D={} engraver", opt.tool, opt.step_over),
        opt.rpm,
        opt.coolant,
        &mut file,
    )?;
    generate_flinque(&opt, &mut file)?;
    trailer(&mut file)?;

    file.flush()
}
