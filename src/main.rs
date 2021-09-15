/// G-Code generator for cutting simple spur gears on a 4th axis, using an involute gear cutter
use std::fs::{File, OpenOptions};
use std::io::{Result, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "gear_gen", about = "A simple spur gear generator")]
struct Opt {
    /// Gear module, must match cutter module
    #[structopt(short, long, default_value = "1")]
    module: f64,

    /// Number of gear teeth
    #[structopt(short, long)]
    teeth: u32,

    /// Diameter of cutter, in mm
    #[structopt(long, default_value = "50")]
    cutter_dia: f64,

    /// Cutter RPM
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

    /// Width of the gear to cut
    #[structopt(short, long)]
    width: f64,

    /// Max depth to cut, in mm
    #[structopt(long, default_value = "0.5")]
    max_depth: f64,

    /// Output file for the resulting G code
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(long)]
    coolant: bool,
}

fn gcode_comment(file: &mut File, s: &str) -> Result<()> {
    writeln!(file, "({})", s)
}

fn preamble(opt: &Opt, file: &mut File) -> Result<()> {
    // Print out the name as a comment on the first line, if set
    if let Some(name) = &opt.name {
        gcode_comment(file, name)?;
    }
    // Comment with tool information
    gcode_comment(
        file,
        &format!("T{} D={} - gear mill", opt.tool, opt.cutter_dia),
    )?;

    // Preamble to set the machine into a reasonable mode
    let preamble_str = "
G90 (Absolute)
G54 (G54 Datum)
G17 (X-Y Plane)
G40 (No cutter compensation)
G80 (No cycles)
G94 (Feed per minute)
G91.1 (Arc absolute mode)
G49 (No tool length compensation)
M9 (Coolant off)

G21 (Metric)

G30 (Go Home Before Starting)
    ";
    write!(file, "{}\n\n", preamble_str)?;
    // Print the tool mode preamble, choosing the tool,
    // enabling length compensation,
    // and executing the tool change cycle
    writeln!(file, "T{} G43 H{} M6", opt.tool, opt.tool)?;

    // Print the Speed preamble, and turn on the spindle
    writeln!(file, "S{} M3", opt.rpm)?;

    // If chosen, start coolant flowing
    if opt.coolant {
        writeln!(file, "M8")?;
    }

    Ok(())
}

fn trailer(_opt: &Opt, file: &mut File) -> Result<()> {
    writeln!(file, "M9 (Coolant off)")?;
    writeln!(file, "M5 (Spindle off)")?;
    writeln!(file, "M30")?;

    Ok(())
}

fn pass_at_depth(opt: &Opt, file: &mut File, depth: f64) -> Result<()> {
    // Clearance (in mm) away from the stock where we move at feed rate
    let clearance = 4.0;

    let clearance_theta = (2.0 * clearance / opt.cutter_dia).asin();
    let x_clearance = (opt.cutter_dia / 2.0) * clearance_theta.tan();

    let y_pos = (opt.teeth as f64 + 2.0) * opt.module / 2.0 // Stock radius
        + opt.cutter_dia / 2.0 // Plus cutter radius
        - depth; // Minus depth of cut
    gcode_comment(file, &format!("Pass at depth {}", depth))?;
    // Rapid to our starting point, to the right of the stock
    writeln!(file, "G0 X{:.4} Y{}", x_clearance, y_pos)?;
    writeln!(file, "G0 Z0.")?;

    // Feed into the stock, cutting as we go
    writeln!(file, "G1 X{} F{}", -opt.width, opt.feed)?;

    // Feed out of the stock, moving in Y
    // TODO: This feed-out should probably be radiused, to avoid backlash issues
    writeln!(file, "G1 Y{} F{}", y_pos + clearance, opt.feed)?;
    // Then rapid a little bit straight out before we do the cross move
    writeln!(file, "G0 Y{}", y_pos + clearance + 10.0)?;

    // Go back to where we started, in two moves, first X then Y to make sure we have enough clearance
    writeln!(file, "G0 X{:.4}", x_clearance)?;
    writeln!(file, "G0 Y{}", y_pos)?;

    Ok(())
}

fn cut_tooth(opt: &Opt, file: &mut File, angle: f64) -> Result<()> {
    // First, turn the rotary axis to the right angle, rapid
    writeln!(file, "G0 A{:.4}", angle)?;

    // Total depth varies from source to source.
    // Here, I'm using the formula from the Machinery's Handbook, 31st Edition, "Module System Gear Design"
    let total_depth = 2.157 * opt.module;

    let mut depth = 0.0;

    // Take passes until we've consumed the whole depth.
    while depth < total_depth {
        let remaining = total_depth - depth;
        if remaining > 2.0 * opt.max_depth {
            // Make max_depth passes until we're within 2*max_depth of the final depth
            depth += opt.max_depth;
            pass_at_depth(opt, file, depth)?;
        } else {
            // Then finish off with two equal passes of the remaining depth
            depth += remaining / 2.0;
            pass_at_depth(opt, file, depth)?;
            depth += remaining / 2.0;
            pass_at_depth(opt, file, total_depth)?;
        }
    }

    // Go home between teeth
    write!(file, "G30\n\n")?;

    Ok(())
}

fn cut_teeth(opt: &Opt, file: &mut File) -> Result<()> {
    let tooth_angle = 360.0 / opt.teeth as f64;

    for i in 0..opt.teeth {
        gcode_comment(file, &format!("Tooth {} of {}", i + 1, opt.teeth))?;
        cut_tooth(opt, file, i as f64 * tooth_angle)?;
    }

    Ok(())
}

fn help_text(opt: &Opt) {
    println!(
        "Before cut:
        - Create stock with OD {}mm
        - Set home to center of right face of stock",
        (opt.teeth + 2) as f64 * opt.module
    )
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    help_text(&opt);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&opt.output)?;

    preamble(&opt, &mut file)?;
    cut_teeth(&opt, &mut file)?;
    trailer(&opt, &mut file)?;

    Ok(())
}
