//! G-Code generator for cutting simple spur gears on a 4th axis, using an involute gear cutter
use gcode::{gcode_comment, preamble, trailer};
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

fn pass_at_depth(opt: &Opt, file: &mut File, depth: f64) -> Result<()> {
    // Clearance (in mm) away from the stock where we move at feed rate
    let clearance = 4.0;

    let clearance_theta = (1.0 - 2.0 * clearance / opt.cutter_dia).acos();
    let x_clearance = (opt.cutter_dia / 2.0) * clearance_theta.tan();

    let y_pos = (opt.teeth as f64 + 2.0) * opt.module / 2.0 // Stock radius
        + opt.cutter_dia / 2.0 // Plus cutter radius
        - depth; // Minus depth of cut
    gcode_comment(file, &format!("Pass at depth {depth}"))?;
    // Rapid to our starting point, to the right of the stock
    writeln!(file, "G0 X{x_clearance:.4} Y{y_pos}")?;
    writeln!(file, "G0 Z0.")?;

    // Feed into the stock, cutting as we go
    writeln!(file, "G1 X{} F{}", -opt.width, opt.feed)?;

    // Feed out of the stock, moving in Y
    // TODO: This feed-out should probably be radiused, to avoid backlash issues
    writeln!(file, "G1 Y{} F{}", y_pos + clearance, opt.feed)?;
    // Then rapid a little bit straight out before we do the cross move
    writeln!(file, "G0 Y{}", y_pos + clearance + 10.0)?;

    // Go back to where we started, in two moves, first X then Y to make sure we have enough clearance
    writeln!(file, "G0 X{x_clearance:.4}")?;
    writeln!(file, "G0 Y{y_pos}")?;

    Ok(())
}

fn cut_tooth(opt: &Opt, file: &mut File, angle: f64) -> Result<()> {
    // First, turn the rotary axis to the right angle, rapid
    writeln!(file, "G0 A{angle:.4}")?;

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

    Ok(())
}

fn cut_teeth(opt: &Opt, file: &mut File) -> Result<()> {
    let tooth_angle = 360.0 / opt.teeth as f64;

    for i in 0..opt.teeth {
        gcode_comment(file, &format!("Tooth {} of {}", i + 1, opt.teeth))?;
        cut_tooth(opt, file, i as f64 * tooth_angle)?;
    }

    // Go home at the end
    write!(file, "G30\n\n")?;

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

    preamble(
        &opt.name,
        opt.tool,
        &format!("T{} D={} - gear mill", opt.tool, opt.cutter_dia),
        opt.rpm,
        opt.coolant,
        &mut file,
    )?;
    cut_teeth(&opt, &mut file)?;
    trailer(&mut file)?;

    Ok(())
}
