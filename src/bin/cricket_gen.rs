use anyhow::Result;
use core::f64;
use gcode::fonts::Font;
use gcode::{
    a, af, g0, g1, gcode_comment, inv_feed_g93, preamble, standard_feed_g94, tool_change, trailer,
    xf, xy, xyf, xyza, xyzf, yf, zaf, zf,
};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

const DEG_30: f64 = f64::consts::PI / 6.0;

#[derive(Debug, StructOpt)]
#[structopt(name = "cricket", about = "Makes cricket dice")]
struct Opt {
    #[structopt(long, default_value = "9.52")]
    stock_rad: f64,

    #[structopt(long, default_value = "40.0")]
    dice_len: f64,

    #[structopt(long, default_value = "0.25")]
    /// Depth for engraving
    depth: f64,

    /// Tool RPM
    #[structopt(long, default_value = "8000")]
    rpm: f64,

    /// Cutting feed rate, in mm/min
    #[structopt(long, default_value = "1000")]
    cutting_feed: f64,

    /// Engraving feed rate, in mm/min
    #[structopt(long, default_value = "300")]
    engraving_feed: f64,

    /// Name for the job
    #[structopt(short, long)]
    name: Option<String>,

    /// Tool number for the hexagonal cut
    #[structopt(long, default_value = "15")]
    cutting_tool: u32,

    /// Tool number for the engraving cuts
    #[structopt(long, default_value = "17")]
    engraving_tool: u32,

    /// Tool number for chamfer cuts
    #[structopt(long, default_value = "9")]
    chamfer_tool: u32,

    /// Cutting tool width
    #[structopt(long, default_value = "6.35")]
    cutting_tool_dia: f64,

    #[structopt(long)]
    coolant: bool,
}

struct HexGeom {
    z_depth: f64,
    chord_len: f64,
    y_cut_width: f64,
}

fn calc_hex_geom(opt: &Opt) -> HexGeom {
    let chord_len = 2.0 * opt.stock_rad * DEG_30.sin();
    let z_depth = opt.stock_rad * (1.0 - DEG_30.cos()) + 0.1;
    let y_cut_width = (chord_len + opt.cutting_tool_dia) / 2.0 + 1.0;
    HexGeom {
        z_depth,
        chord_len,
        y_cut_width,
    }
}

// Turn the hexagon into a round, and return the z offset of the resulting flat surfaces
fn make_hexagon_from_round(file: &mut dyn Write, opt: &Opt, geom: &HexGeom) -> Result<()> {
    let z_safe = 1.0;

    // First, go to a safe y and z and bring the A to zero
    g0(file, xyza(0.0, geom.y_cut_width, z_safe, 0.0))?;

    gcode_comment(file, &format!("Faces at depth {}", geom.z_depth))?;
    // Cut the faces
    for face in 0..6 {
        // Go to the right face angle
        g0(file, a(60.0 * face as f64))?;
        // Feed in to cutting z
        g1(file, zf(-geom.z_depth, opt.cutting_feed))?;
        // Calculate the number of passes, and the stepover per pass, slightly less than half the tool width
        let passes = (2.5 * opt.dice_len / opt.cutting_tool_dia).ceil();
        let pass_step = opt.dice_len / passes;
        let mut x = 0.0;
        for pass in 0..passes as usize {
            g1(file, xf(x, opt.cutting_feed))?;
            if pass % 2 == 0 {
                g1(file, yf(-geom.y_cut_width, opt.cutting_feed))?;
            } else {
                g1(file, yf(geom.y_cut_width, opt.cutting_feed))?;
            }
            x -= pass_step;
        }
        // Feed out to safe z
        g1(file, zf(z_safe, opt.cutting_feed))?;
    }

    // Now knock the corners off to make the dice roll nicer
    let corner_depth = geom.z_depth / 3.5;
    gcode_comment(file, &format!("Corner chamfer at depth {corner_depth}"))?;

    for corner in 0..6 {
        g0(file, xy(opt.cutting_tool_dia, 0.0))?;
        g0(file, a(60.0 * corner as f64 + 30.0))?;
        g1(file, zf(-corner_depth, opt.cutting_feed))?;
        g1(
            file,
            xyf(
                -(opt.dice_len - opt.cutting_tool_dia / 2.5),
                0.0,
                opt.cutting_feed,
            ),
        )?;
        g1(file, zf(z_safe, opt.cutting_feed))?;
    }
    Ok(())
}

fn chamfer(file: &mut dyn Write, opt: &Opt, geom: &HexGeom) -> Result<()> {
    let z_safe = 1.0;
    let chamfer_offset = opt.cutting_tool_dia / 4.0;
    let chamfer_edge_width = 1.0;
    let chamfer_depth = -geom.z_depth - chamfer_offset - chamfer_edge_width * 2.0_f64.sqrt();
    // Tool change to the chamfering tool
    tool_change(file, opt.chamfer_tool, opt.rpm)?;
    // First, go to a safe y and z and bring the A to zero
    g0(file, xyza(0.0, geom.y_cut_width, z_safe, 0.0))?;
    for face in 0..6 {
        gcode_comment(
            file,
            &format!("Chamfer side {face} at offset {chamfer_offset} and depth {chamfer_depth}"),
        )?;
        g0(file, a(60.0 * face as f64))?;

        g0(file, xy(chamfer_offset, geom.y_cut_width))?;
        // Feed in to cutting z
        g1(file, zf(chamfer_depth, opt.cutting_feed))?;
        // Cut along y- direction
        g1(file, yf(-geom.y_cut_width, opt.cutting_feed))?;
        // Feed out to safe z
        g1(file, zf(z_safe, opt.cutting_feed))?;

        // Rapid to the other end
        g0(file, xy(-opt.dice_len + chamfer_offset, geom.y_cut_width))?;
        // Feed in to cutting z
        g1(file, zf(chamfer_depth, opt.cutting_feed))?;
        // Cut along y- direction
        g1(file, yf(-geom.y_cut_width, opt.cutting_feed))?;
        // Feed out to safe z
        g1(file, zf(z_safe, opt.cutting_feed))?;
    }
    Ok(())
}

fn offcut(file: &mut dyn Write, opt: &Opt) -> Result<()> {
    // Circular movement feed is based on the inverse feed rate mode, in units of 1/minutes
    let stock_circumference = opt.stock_rad * f64::consts::TAU;
    let cutting_time_minutes = stock_circumference / opt.cutting_feed;
    let inv_feed = 1.0 / cutting_time_minutes;

    let safe_z = 1.0;

    // Cut down to just over half way through the stock
    let cutting_radius = opt.stock_rad * 0.6;
    let cut_pass_depth = 1.0;
    let passes = (cutting_radius / cut_pass_depth).ceil();

    // Back to the cutting tool
    tool_change(file, opt.cutting_tool, opt.rpm)?;
    // First, rapid to our cutting X
    g0(file, xyza(-opt.dice_len, 0.0, safe_z, 0.0))?;

    for pass in 0..passes as usize {
        // Feed in to the starting cutting Z for the pass
        let start_cutting_z = pass as f64 * cutting_radius / passes;
        let end_cutting_z = (pass + 1) as f64 * cutting_radius / passes;
        gcode_comment(
            file,
            &format!("Offcut pass {pass} from Z {start_cutting_z} to Z {end_cutting_z}"),
        )?;
        // Feed in to the start cutting Z of this radial cut
        g1(file, zf(-start_cutting_z, opt.cutting_feed))?;
        // Now do the rotary move in inverse feed rate mode, going down to the next depth and turining at the same time
        inv_feed_g93(file)?;
        g1(file, zaf(-end_cutting_z, 360.0 * pass as f64, inv_feed))?;
        standard_feed_g94(file)?;
    }
    gcode_comment(
        file,
        &format!("Offcut final rotary pass at Z {cutting_radius}"),
    )?;
    // Finishing rotary pass at full depth
    inv_feed_g93(file)?;
    g1(file, af(360.0 * passes as f64, inv_feed))?;
    standard_feed_g94(file)?;
    // And feed out to safe z
    g1(file, zf(safe_z, opt.cutting_feed))?;

    Ok(())
}

fn engrave_text_on_hex(
    file: &mut dyn Write,
    text: &[&str],
    opt: &Opt,
    geom: &HexGeom,
    font: &Font,
) -> Result<()> {
    assert!(text.len() == 6);
    let z_safe = 1.0;
    let y_safe = geom.chord_len / 2.0 + 1.0;
    let font_scale = geom.chord_len / 1.3;
    tool_change(file, opt.engraving_tool, opt.rpm)?;
    // First, go to a safe y and z and bring the A to zero
    g0(file, xyza(0.0, y_safe, z_safe, 0.0))?;
    for (i, line) in text.iter().enumerate() {
        // Get the line width
        let str_len = font.string_len(line) * font_scale;
        println!("{line} len {str_len}");
        assert!(str_len < opt.dice_len);
        // Calculate the x and y offsets to get the string nicely centered
        let x_off = -(opt.dice_len + str_len - opt.cutting_tool_dia / 2.0) / 2.0;
        let y_off = -font.t_height * font_scale / 2.0;
        // Go to the correct A angle
        g0(file, a(60.0 * i as f64))?;
        // Now engrave the string, in two passes
        font.string_to_gcode(
            file,
            line,
            &xyzf(
                x_off,
                y_off,
                -geom.z_depth - opt.depth / 2.0,
                opt.engraving_feed,
            ),
            z_safe,
            font_scale,
        )?;
        font.string_to_gcode(
            file,
            line,
            &xyzf(x_off, y_off, -geom.z_depth - opt.depth, opt.engraving_feed),
            z_safe,
            font_scale,
        )?;
    }

    Ok(())
}

fn make_cricket_dice(filename: &str, text: &[&str], opt: &Opt, font: &Font) -> Result<()> {
    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)?,
    );

    preamble(
        &opt.name,
        opt.cutting_tool,
        &format!("T{} D={} end mill", opt.cutting_tool, opt.cutting_tool_dia),
        opt.rpm,
        opt.coolant,
        &mut file,
    )?;

    let geom = calc_hex_geom(opt);

    make_hexagon_from_round(&mut file, opt, &geom)?;
    engrave_text_on_hex(&mut file, text, opt, &geom, font)?;
    offcut(&mut file, opt)?;
    chamfer(&mut file, opt, &geom)?;
    trailer(&mut file)?;

    file.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let font = Font::new_from_svg(&PathBuf::from_str("EMSReadability.svg")?)?;
    let runs_text = &["ONE", "TWO", "THREE", "FOUR", "SIX", "HOWZAT!"];
    make_cricket_dice("cricket_runs.nc", runs_text, &opt, &font)?;
    let wicket_text = &["RUN OUT", "NOT OUT!", "CAUGHT", "STUMPED", "BOWLED", "LBW"];
    make_cricket_dice("cricket_wicket.nc", wicket_text, &opt, &font)?;
    Ok(())
}
