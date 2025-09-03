use core::f64;
///! G-Code generator for a kind of wavy spiral guilloche
use gcode::{g0, g1, gcode_comment, preamble, trailer, xyz, xyzf, zf};
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

    #[structopt(long, default_value = "0.5")]
    /// Width of each turn in the spiral
    pass_width: f64,

    #[structopt(long, default_value = "0.1")]
    /// Minimum depth (i.e. the shallowest the tool will go in negative Z)
    min_depth: f64,

    #[structopt(long, default_value = "0.4")]
    /// Minimum depth (i.e. the deepest the tool will go in negative Z)
    max_depth: f64,

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

fn generate_spiral(opt: &Opt, file: &mut dyn Write, z_off: f64) -> Result<()> {
    let turns = (opt.outer_rad / opt.pass_width).floor() as usize;
    let skip_turns = (opt.inner_rad / opt.pass_width).ceil() as usize;

    let start_x = skip_turns as f64 * opt.pass_width;

    gcode_comment(file, &format!("Pass at offset {z_off}mm"))?;
    println!("Pass at z offset {z_off}mm");
    // Rapid the starting position
    g0(file, xyz(start_x, 0., 10.0))?;
    g0(file, xyz(start_x, 0., 1.0 + z_off))?;
    // Then slowly plunge to depth
    g1(
        file,
        xyzf(start_x, 0., z_off - opt.min_depth, opt.feed / 3.0),
    )?;

    for turn in skip_turns..turns {
        gcode_comment(file, &format!("Turn: {turn}"))?;
        for angle_step in 0..opt.steps_per_turn {
            let circle_progress = angle_step as f64 / opt.steps_per_turn as f64;
            let angle = 2.0 * f64::consts::PI * circle_progress;
            // In range [0, 1], where we are on the z cycle
            let z_step = (1.0 + (angle * opt.rays as f64).sin()) / 2.0;
            let z = (opt.max_depth - opt.min_depth) * z_step + opt.min_depth;

            let radius =
                (circle_progress + turn as f64) * opt.pass_width + z_step * opt.radial_wobble;
            let x = radius * angle.cos();
            let y = radius * angle.sin();

            g1(file, xyzf(x, y, z_off - z, opt.feed))?;
        }
    }
    // Feed out
    g1(file, zf(1.0, opt.feed))?;
    Ok(())
}

fn generate_spiral_step_down(opt: &Opt, file: &mut dyn Write) -> Result<()> {
    let steps = (opt.max_depth / opt.max_stepdown).ceil() as usize;
    for step in 1..(steps + 1) {
        let z_off = opt.max_depth * (1.0 - step as f64 / steps as f64);
        generate_spiral(opt, file, z_off)?;
    }
    Ok(())
}

fn help_text(opt: &Opt) {
    let spiral_length = f64::consts::PI * opt.outer_rad.powf(2.0) / (2.0 * opt.pass_width);
    let steps = (opt.max_depth / opt.max_stepdown).ceil();
    println!(
        "Before cut:
        - Create stock with diameter at least {}mm
        - Set home to center of stock, at the top,
        - Spiral length {}mm,
        - {} steps,
        - Approx run time {} minutes",
        opt.outer_rad * 2.0,
        spiral_length,
        steps,
        steps * spiral_length / opt.feed,
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
        &format!("T{} D={} engraver", opt.tool, opt.pass_width),
        opt.rpm,
        opt.coolant,
        &mut file,
    )?;
    generate_spiral_step_down(&opt, &mut file)?;
    trailer(&mut file)?;

    file.flush()
}
