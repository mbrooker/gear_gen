use anyhow::Result;
use core::f64;
use gcode::fonts::Font;
use gcode::{a, g0, g1, gcode_comment, preamble, tool_change, trailer, xf, xyz, xyza, xyzf, yf, zf};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

const DEG_30: f64 = f64::consts::PI / 6.0;
const DEG_60: f64 = f64::consts::PI / 2.0;

#[derive(Debug, StructOpt)]
#[structopt(name = "cricket", about = "Makes cricket dice")]
struct Opt {
    #[structopt(long, default_value = "9.52")]
    stock_rad: f64,

    #[structopt(long, default_value = "40.0")]
    dice_len: f64,

    #[structopt(long, default_value = "0.1")]
    /// Depth for engraving
    depth: f64,

    /// Tool RPM
    #[structopt(long, default_value = "8000")]
    rpm: f64,

    /// Feed rate, in mm/min
    #[structopt(long, default_value = "300")]
    feed: f64,

    /// Name for the job
    #[structopt(short, long)]
    name: Option<String>,

    /// Tool number for the hexagonal cut
    #[structopt(long, default_value = "15")]
    cutting_tool: u32,

    /// Tool number for the engraving cuts
    #[structopt(long, default_value = "17")]
    engraving_tool: u32,

    /// Cutting tool width
    #[structopt(long, default_value = "6.35")]
    cutting_tool_dia: f64,

    /// Output file for the resulting G code
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(long)]
    coolant: bool,
}

struct HexGeom {
    z_depth: f64,
    chord_len: f64,
}

// Turn the hexagon into a round, and return the z offset of the resulting flat surfaces
fn make_hexagon_from_round(file: &mut dyn Write, opt: &Opt) -> Result<HexGeom> {
    let chord_len = 2.0 * opt.stock_rad * DEG_30.sin();
    let y_cut_width = (chord_len + opt.cutting_tool_dia) / 2.0 + 1.0;
    let z_safe = 1.0;
    let z_depth = opt.stock_rad * (1.0 - DEG_30.cos()) + 0.1;

    // First, go to a safe y and z and bring the A to zero
    g0(file, xyza(0.0, y_cut_width, z_safe, 0.0))?;

    for face in 0..6 {
        // Go to the right face angle
        g0(file, a(60.0 * face as f64))?;
        // Feed in to cutting z
        g1(file, zf(-z_depth, opt.feed))?;
        // Calculate the number of passes, and the stepover per pass, slightly less than half the tool width
        let passes = (2.5 * opt.dice_len / opt.cutting_tool_dia).ceil();
        let pass_step = opt.dice_len / passes;
        let mut x = 0.0;
        for pass in 0..passes as usize {
            g1(file, xf(x, opt.feed))?;
            if pass % 2 == 0 {
                g1(file, yf(-y_cut_width, opt.feed))?;
            } else {
                g1(file, yf(y_cut_width, opt.feed))?;
            }
            x -= pass_step;
        }
        // Feed out to safe z
        g1(file, zf(z_safe, opt.feed))?;
    }

    Ok(HexGeom { z_depth, chord_len })
}

fn engrave_text_on_hex(
    file: &mut dyn Write,
    text: &[&str],
    opt: &Opt,
    geom: HexGeom,
    font: &Font,
) -> Result<()> {
    assert!(text.len() == 6);
    let z_safe = 1.0;
    let y_safe = geom.chord_len / 2.0 + 1.0;
    let font_scale = geom.chord_len / 1.5;
    // First, go to a safe y and z and bring the A to zero
    g0(file, xyza(0.0, y_safe, z_safe, 0.0))?;
    for (i, line) in text.into_iter().enumerate() {
        // Get the line width
        let str_len = font.string_len(line) * font_scale;
        println!("{line} len {str_len}");
        assert!(str_len < opt.dice_len);
        // Calculate the x and y offsets to get the string nicely centered
        let x_off = -(opt.dice_len + str_len) / 2.0;
        let y_off = -font.ascent * font_scale / 2.0;
        // Go to the correct A angle
        g0(file, a(60.0 * i as f64))?;
        // Now engrave the string
        font.string_to_gcode(
            file,
            line,
            &xyzf(x_off, y_off, -geom.z_depth - opt.depth, opt.feed),
            z_safe,
            font_scale,
        )?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&opt.output)?,
    );

    preamble(
        &opt.name,
        opt.cutting_tool,
        &format!("T{} D={} end mill", opt.cutting_tool, opt.cutting_tool_dia),
        opt.rpm,
        opt.coolant,
        &mut file,
    )?;
    let font = Font::new_from_svg(&PathBuf::from_str("EMSReadability.svg")?)?;
    let geom = make_hexagon_from_round(&mut file, &opt)?;
    let text = &["ONE", "TWO", "THREE", "FOUR", "SIX", "HOWZAT!"];
    tool_change(&mut file, opt.engraving_tool, opt.rpm)?;
    engrave_text_on_hex(&mut file, text, &opt, geom, &font)?;
    trailer(&mut file)?;

    file.flush()?;
    Ok(())
}
