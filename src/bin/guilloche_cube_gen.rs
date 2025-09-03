use core::f64;
///! G-Code generator for a kind of wavy spiral guilloche
use gcode::{g0, g1, gcode_comment, preamble, trailer, xyf, xyz, zf};
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
    #[structopt(long, default_value = "19")]
    /// Outer radius
    outer_rad: f64,

    #[structopt(long, default_value = "0.2")]
    /// Cut depth
    depth: f64,

    #[structopt(long, default_value = "0.8")]
    /// Step over for each line
    step_over: f64,

    #[structopt(long, default_value = "4.0")]
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
    #[structopt(long, default_value = "17")]
    tool: u32,

    /// Output file for the resulting G code
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(long)]
    coolant: bool,
}

fn generate_cube(opt: &Opt, file: &mut dyn Write, cx: f64, cy: f64) -> Result<()> {
    let safe_z = 1.0;
    let steps = (opt.cube_size / opt.step_over).floor() as usize + 1;
    let y_adv = opt.cube_size * DEG_30.sin();
    let x_adv = opt.cube_size * DEG_30.cos();
    gcode_comment(file, &format!("Cube at {cx}, {cy}"))?;
    for i in 0..steps {
        let base_y = cy - i as f64 * opt.step_over;

        g0(file, xyz(cx - x_adv, base_y + y_adv, safe_z))?;
        g1(file, zf(-opt.depth, opt.feed))?;
        g1(file, xyf(cx, base_y, opt.feed))?;
        g1(file, xyf(cx + x_adv, base_y + y_adv, opt.feed))?;
        g1(file, zf(safe_z, opt.feed))?;
    }

    Ok(())
}

fn generate_cubes(opt: &Opt, file: &mut dyn Write) -> Result<()> {
    let width = 2.0 * DEG_30.cos() * opt.cube_size;
    let height = opt.cube_size * (1.0 + DEG_30.sin());
    let nx = 2 * (opt.outer_rad / width).ceil() as usize;
    let ny = (opt.outer_rad / opt.cube_size) as usize;
    for y in 0..ny {
        let cy = y as f64 * height;
        gcode_comment(file, &format!("Row {y} at {cy}"))?;
        for x in 0..nx {
            let cx = x as f64 * width + (y % 2) as f64 * width / 2.0;

            generate_cube(opt, file, cx, cy)?;
        }
    }
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
    trailer(&mut file)?;

    file.flush()
}
