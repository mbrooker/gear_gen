//! G-Code generator for a kind of wavy spiral guilloche
//!
use core::f64;
use gcode::{
    gcode_comment, patterns, preamble, trailer, trimmed_g1_path, xy, xyf, xyr,
    PosRadiusAndFeed,
};
use std::fs::OpenOptions;
use std::io::{BufWriter, Result, Write};
use std::path::PathBuf;
use structopt::StructOpt;

const DEG_30: f64 = f64::consts::PI / 6.0;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "cube_gen",
    about = "Generates a kind of guilloche-inspired pattern of QBert cubes"
)]
struct Opt {
    #[structopt(long, default_value = "16")]
    /// Outer radius
    outer_rad: f64,

    #[structopt(long, default_value = "0.1")]
    /// Cut depth
    depth: f64,

    #[structopt(long, default_value = "0.5")]
    /// Step over for each line
    step_over: f64,

    #[structopt(long, default_value = "2.5")]
    /// Size of each cube
    cube_size: f64,

    /// Tool RPM
    #[structopt(long, default_value = "8000")]
    rpm: f64,

    /// Feed rate, in mm/min
    #[structopt(long, default_value = "300")]
    feed: f64,

    /// Name for the job
    #[structopt(short, long)]
    name: Option<String>,

    /// Tool number for the cut
    #[structopt(long, default_value = "23")]
    tool: u32,

    /// Output file for the resulting G code
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(long)]
    coolant: bool,
}

fn generate_cube(
    opt: &Opt,
    file: &mut dyn Write,
    cx: f64,
    cy: f64,
    trimmer: &PosRadiusAndFeed,
) -> Result<()> {
    let safe_z = 1.0;
    let steps = (opt.cube_size / opt.step_over).floor() as usize + 1;
    let y_adv = opt.cube_size * DEG_30.sin();
    let x_adv = opt.cube_size * DEG_30.cos();
    gcode_comment(file, &format!("Cube at {cx}, {cy}"))?;
    for i in 0..steps {
        let base_y = cy - i as f64 * opt.step_over;

        trimmed_g1_path(
            file,
            safe_z,
            -opt.depth,
            opt.feed,
            &[
                xy(cx - x_adv, base_y + y_adv),
                xy(cx, base_y),
                xy(cx + x_adv, base_y + y_adv),
            ],
            trimmer,
        )?;
    }

    Ok(())
}

fn generate_cubes(opt: &Opt, file: &mut dyn Write) -> Result<()> {
    let width = 2.0 * DEG_30.cos() * opt.cube_size;
    let height = opt.cube_size * (1.0 + DEG_30.sin());
    let nx = 2 * (opt.outer_rad / width).ceil() as usize;
    let ny = 2 * (opt.outer_rad / opt.cube_size) as usize;
    for y in 0..ny {
        let cy = y as f64 * height - opt.outer_rad;
        gcode_comment(file, &format!("Row {y} at {cy}"))?;
        for x in 0..nx {
            let cx = x as f64 * width + (y % 2) as f64 * width / 2.0 - opt.outer_rad;

            generate_cube(opt, file, cx, cy, &xyr(0.0, 0.0, opt.outer_rad * 0.82))?;
        }
    }
    Ok(())
}

fn tick_marks(opt: &Opt, file: &mut dyn Write) -> Result<()> {
    let outer_rad = opt.outer_rad;
    let center = xyf(0.0, 0.0, opt.feed);

    patterns::radial_tick_marks(
        file,
        opt.outer_rad * 0.92,
        outer_rad,
        60,
        &center,
        -opt.depth,
        &[5],
    )?;

    patterns::radial_tick_segments(
        file,
        opt.outer_rad * 0.88,
        outer_rad,
        12,
        &center,
        -opt.depth,
        f64::consts::TAU / 180.0,
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
        &format!("T{} D={} engraver", opt.tool, opt.step_over),
        opt.rpm,
        opt.coolant,
        &mut file,
    )?;
    generate_cubes(&opt, &mut file)?;
    tick_marks(&opt, &mut file)?;
    trailer(&mut file)?;

    file.flush()
}
