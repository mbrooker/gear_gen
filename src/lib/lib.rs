use std::fs::File;
use std::io::{Result, Write};

pub fn gcode_comment(file: &mut File, s: &str) -> Result<()> {
    writeln!(file, "({})", s)
}

pub fn trailer(file: &mut File) -> Result<()> {
    writeln!(file, "M9 (Coolant off)")?;
    writeln!(file, "M5 (Spindle off)")?;
    writeln!(file, "M30")?;

    Ok(())
}

pub fn preamble(
    name: &Option<String>,
    tool: u32,
    cutter_dia: f64,
    rpm: f64,
    coolant: bool,
    file: &mut File,
) -> Result<()> {
    // Print out the name as a comment on the first line, if set
    if let Some(name) = &name {
        gcode_comment(file, name)?;
    }
    // Comment with tool information
    gcode_comment(file, &format!("T{} D={} - gear mill", tool, cutter_dia))?;

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
    writeln!(file, "T{} G43 H{} M6", tool, tool)?;

    // Print the Speed preamble, and turn on the spindle
    writeln!(file, "S{} M3", rpm)?;

    // If chosen, start coolant flowing
    if coolant {
        writeln!(file, "M8")?;
    }

    Ok(())
}

pub struct PosAndFeed {
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    a: Option<f64>,
    feed: Option<f64>,
}

pub fn x(x: f64) -> PosAndFeed {
    PosAndFeed {
        x: Some(x),
        y: None,
        z: None,
        a: None,
        feed: None,
    }
}

pub fn xy(x: f64, y: f64) -> PosAndFeed {
    PosAndFeed {
        x: Some(x),
        y: Some(y),
        z: None,
        a: None,
        feed: None,
    }
}

pub fn xyza(x: f64, y: f64, z: f64, a: f64) -> PosAndFeed {
    PosAndFeed {
        x: Some(x),
        y: Some(y),
        z: Some(z),
        a: Some(a),
        feed: None,
    }
}

pub fn zf(z: f64, feed: f64) -> PosAndFeed {
    PosAndFeed {
        x: None,
        y: None,
        z: Some(z),
        a: None,
        feed: Some(feed),
    }
}

pub fn zaf(z: f64, a: f64, feed: f64) -> PosAndFeed {
    PosAndFeed {
        x: None,
        y: None,
        z: Some(z),
        a: Some(a),
        feed: Some(feed),
    }
}

/// Emit a gcode parameter value, if `ov` is `Some`.
/// To make the gcode human-friendly, numbers that round nicely are printed in their minimal form.
fn g_val(file: &mut File, name: &str, ov: Option<f64>) -> Result<()> {
    if let Some(v) = ov {
        if v == v.round() {
            write!(file, " {}{}.", name, v)
        } else {
            write!(file, " {}{:.4}", name, v)
        }
    } else {
        Ok(())
    }
}

fn g_cmd(file: &mut File, g: &str, p: PosAndFeed) -> Result<()> {
    if p.x.is_none() && p.y.is_none() && p.z.is_none() {
        panic!("Refusing to make illegal {}", g);
    }
    write!(file, "{}", g)?;
    g_val(file, "X", p.x)?;
    g_val(file, "Y", p.y)?;
    g_val(file, "Z", p.z)?;
    g_val(file, "A", p.a)?;
    g_val(file, "F", p.feed)?;
    writeln!(file)?;
    Ok(())
}

pub fn g0(file: &mut File, p: PosAndFeed) -> Result<()> {
    g_cmd(file, "G0", p)
}

pub fn g1(file: &mut File, p: PosAndFeed) -> Result<()> {
    g_cmd(file, "G1", p)
}
