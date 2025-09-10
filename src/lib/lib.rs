use std::io::{Result, Write};

use crate::geometry::{trim, Circle, LineSegment};
pub mod fonts;
mod geometry;
pub mod patterns;

pub fn gcode_comment(file: &mut dyn Write, s: &str) -> Result<()> {
    writeln!(file, "({s})")
}

pub fn trailer(file: &mut dyn Write) -> Result<()> {
    writeln!(file, "G30 (Go Home)")?;
    writeln!(file, "M9 (Coolant off)")?;
    writeln!(file, "M5 (Spindle off)")?;
    writeln!(file, "M30")?;

    Ok(())
}

pub fn preamble(
    name: &Option<String>,
    tool: u32,
    tool_comment: &str,
    rpm: f64,
    coolant: bool,
    file: &mut dyn Write,
) -> Result<()> {
    // Print out the name as a comment on the first line, if set
    if let Some(name) = &name {
        gcode_comment(file, name)?;
    }
    // Comment with tool information
    gcode_comment(file, tool_comment)?;

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
    write!(file, "{preamble_str}\n\n")?;
    tool_change(file, tool, rpm)?;

    // If chosen, start coolant flowing
    if coolant {
        writeln!(file, "M8")?;
    }

    Ok(())
}

pub fn tool_change(file: &mut dyn Write, tool: u32, rpm: f64) -> Result<()> {
    // First, turn off the spindle
    writeln!(file, "M5 (Spindle off)")?;
    // Go home
    writeln!(file, "G30 (Go Home)")?;
    // Then do a stop for the user to change the tool
    writeln!(file, "M0 (stop for tool change)")?;
    // Print the tool mode preamble, choosing the tool,
    // enabling length compensation,
    // and executing the tool change cycle
    writeln!(file, "T{tool} G43 H{tool} M6")?;

    // Print the Speed preamble, and turn on the spindle
    writeln!(file, "S{rpm} M3")?;

    Ok(())
}

trait AsGVals {
    fn as_gvals(&self, file: &mut dyn Write) -> Result<()>;
}

#[derive(Clone, Debug)]
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

pub fn a(a: f64) -> PosAndFeed {
    PosAndFeed {
        x: None,
        y: None,
        z: None,
        a: Some(a),
        feed: None,
    }
}

pub fn xaf(x: f64, a: f64, feed: f64) -> PosAndFeed {
    PosAndFeed {
        x: Some(x),
        y: None,
        z: None,
        a: Some(a),
        feed: Some(feed),
    }
}

pub fn xf(x: f64, feed: f64) -> PosAndFeed {
    PosAndFeed {
        x: Some(x),
        y: None,
        z: None,
        a: None,
        feed: Some(feed),
    }
}

pub fn yf(y: f64, feed: f64) -> PosAndFeed {
    PosAndFeed {
        x: None,
        y: Some(y),
        z: None,
        a: None,
        feed: Some(feed),
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

pub fn xyz(x: f64, y: f64, z: f64) -> PosAndFeed {
    PosAndFeed {
        x: Some(x),
        y: Some(y),
        z: Some(z),
        a: None,
        feed: None,
    }
}

pub fn xyf(x: f64, y: f64, feed: f64) -> PosAndFeed {
    PosAndFeed {
        x: Some(x),
        y: Some(y),
        z: None,
        a: None,
        feed: Some(feed),
    }
}

pub fn xyzf(x: f64, y: f64, z: f64, feed: f64) -> PosAndFeed {
    PosAndFeed {
        x: Some(x),
        y: Some(y),
        z: Some(z),
        a: None,
        feed: Some(feed),
    }
}

pub fn xzf(x: f64, z: f64, feed: f64) -> PosAndFeed {
    PosAndFeed {
        x: Some(x),
        y: None,
        z: Some(z),
        a: None,
        feed: Some(feed),
    }
}

pub fn z(z: f64) -> PosAndFeed {
    PosAndFeed {
        x: None,
        y: None,
        z: Some(z),
        a: None,
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

impl AsGVals for PosAndFeed {
    fn as_gvals(&self, file: &mut dyn Write) -> Result<()> {
        if self.x.is_none() && self.y.is_none() && self.z.is_none() && self.a.is_none() {
            panic!("Refusing to make illegal move");
        }
        g_val(file, "X", self.x)?;
        g_val(file, "Y", self.y)?;
        g_val(file, "Z", self.z)?;
        g_val(file, "A", self.a)?;
        g_val(file, "F", self.feed)?;
        Ok(())
    }
}

/// Emit a gcode parameter value, if `ov` is `Some`.
/// To make the gcode human-friendly, numbers that round nicely are printed in their minimal form.
fn g_val(file: &mut dyn Write, name: &str, ov: Option<f64>) -> Result<()> {
    if let Some(v) = ov {
        if (v - v.round()).abs() < f64::EPSILON {
            write!(file, " {}{}.", name, v.round())
        } else {
            write!(file, " {name}{v:.4}")
        }
    } else {
        Ok(())
    }
}

fn g_move_linear(file: &mut dyn Write, g: &str, p: &dyn AsGVals) -> Result<()> {
    write!(file, "{g}")?;
    p.as_gvals(file)?;
    writeln!(file)?;
    Ok(())
}

pub fn g0(file: &mut dyn Write, p: PosAndFeed) -> Result<()> {
    assert!(p.feed.is_none(), "g0 moves must not include a feed rate");
    if let Some(z) = p.z {
        assert!(z > 0.0, "Rapid move at negative z");
    }
    g_move_linear(file, "G0", &p)
}

pub fn g1(file: &mut dyn Write, p: PosAndFeed) -> Result<()> {
    assert!(p.feed.is_some(), "g1 moves must include a feed rate");
    g_move_linear(file, "G1", &p)
}

/// Trimmed g1 move. Here, we take a list of point to connect with lines, and emit a set of G1 moves.
///  Only the segments of moves inside `circle` are emitted
pub fn trimmed_g1_path(
    file: &mut dyn Write,
    z_safe: f64,
    z_cut: f64,
    feed: f64,
    path: &[PosAndFeed],
    circle: &PosRadiusAndFeed,
) -> Result<()> {
    let mut cutter_down = false;
    // Make sure the cutter is up
    g0(file, z(z_safe))?;

    let trimmer = &Circle::new(circle);
    for i in 0..(path.len() - 1) {
        let seg = trim(LineSegment::new(&path[i], &path[i + 1]), trimmer);
        let raise_at_end = seg.is_none() || seg.is_trimmed();
        if !seg.is_none() {
            let points = seg.unwrap();
            let p1: PosAndFeed = points.start.into();
            let p2: PosAndFeed = points.end.into();
            if !cutter_down {
                // Rapid to start position
                g0(file, p1)?;
                // Lower the cutter
                g1(file, zf(z_cut, feed))?;
                cutter_down = true;
            }
            // Now cut
            g1(file, xyf(p2.x.unwrap(), p2.y.unwrap(), feed))?;
        }
        if raise_at_end && cutter_down {
            g1(file, zf(z_safe, feed))?;
            cutter_down = false;
        }
    }

    if cutter_down {
        g1(file, zf(z_safe, feed))?;
    }

    Ok(())
}

pub struct PosRadiusAndFeed {
    x: Option<f64>,
    y: Option<f64>,
    r: Option<f64>,
    feed: Option<f64>,
    z: Option<f64>,
}

pub fn xyrf(x: f64, y: f64, r: f64, feed: f64) -> PosRadiusAndFeed {
    PosRadiusAndFeed {
        x: Some(x),
        y: Some(y),
        r: Some(r),
        feed: Some(feed),
        z: None,
    }
}

pub fn xyr(x: f64, y: f64, r: f64) -> PosRadiusAndFeed {
    PosRadiusAndFeed {
        x: Some(x),
        y: Some(y),
        r: Some(r),
        feed: None,
        z: None,
    }
}

pub fn xyzrf(x: f64, y: f64, z: f64, r: f64, feed: f64) -> PosRadiusAndFeed {
    PosRadiusAndFeed {
        x: Some(x),
        y: Some(y),
        r: Some(r),
        feed: Some(feed),
        z: Some(z),
    }
}

impl AsGVals for PosRadiusAndFeed {
    fn as_gvals(&self, file: &mut dyn Write) -> Result<()> {
        g_val(file, "X", self.x)?;
        g_val(file, "Y", self.y)?;
        g_val(file, "R", self.r)?;
        g_val(file, "F", self.feed)?;
        g_val(file, "Z", self.z)?;
        Ok(())
    }
}

pub struct PosXYIJ {
    x: Option<f64>,
    y: Option<f64>,
    i: Option<f64>,
    j: Option<f64>,
    z: Option<f64>,
    feed: Option<f64>,
}

pub fn xyijf(x: f64, y: f64, i: f64, j: f64, feed: f64) -> PosXYIJ {
    PosXYIJ {
        x: Some(x),
        y: Some(y),
        i: Some(i),
        j: Some(j),
        z: None,
        feed: Some(feed),
    }
}

pub fn xyzijf(x: f64, y: f64, z: f64, i: f64, j: f64, feed: f64) -> PosXYIJ {
    PosXYIJ {
        x: Some(x),
        y: Some(y),
        i: Some(i),
        j: Some(j),
        z: Some(z),
        feed: Some(feed),
    }
}

impl AsGVals for PosXYIJ {
    fn as_gvals(&self, file: &mut dyn Write) -> Result<()> {
        g_val(file, "X", self.x)?;
        g_val(file, "Y", self.y)?;
        g_val(file, "I", self.i)?;
        g_val(file, "J", self.j)?;
        g_val(file, "Z", self.z)?;
        g_val(file, "F", self.feed)?;
        Ok(())
    }
}

/// G2 clockwise arc move
/// See https://www.cnccookbook.com/cnc-g-code-arc-circle-g02-g03/ for a good description of what the params mean
/// X, Y is endpoint, I, J is offset from start point to true arc center
pub fn g2(file: &mut dyn Write, p: PosXYIJ) -> Result<()> {
    if p.x.is_none() || p.y.is_none() || p.i.is_none() || p.j.is_none() || p.feed.is_none() {
        panic!("Refusing to make illegal G2 move");
    }
    g_move_linear(file, "G2", &p)
}

/// G3 counter-clockwise arc move
/// See https://www.cnccookbook.com/cnc-g-code-arc-circle-g02-g03/ for a good description of what the params mean
/// X, Y is endpoint, I, J is offset from start point to true arc center
pub fn g3(file: &mut dyn Write, p: PosXYIJ) -> Result<()> {
    if p.x.is_none() || p.y.is_none() || p.i.is_none() || p.j.is_none() || p.feed.is_none() {
        panic!("Refusing to make illegal G2 move");
    }
    g_move_linear(file, "G3", &p)
}

/// Full circle move
pub fn g2_circle(file: &mut dyn Write, center: PosRadiusAndFeed, safe_z: f64) -> Result<()> {
    let x0 = center.x.unwrap() + center.r.unwrap();
    let x1 = center.x.unwrap() - center.r.unwrap();
    let y = center.y.unwrap();
    let feed = center.feed.unwrap();
    g0(file, xyz(x0, y, safe_z))?;
    g1(file, xyzf(x0, y, center.z.unwrap(), feed))?;
    g2(file, xyijf(x0, y, x1, y, feed))?;
    g1(file, zf(safe_z, feed))?;
    Ok(())
}

/// Helical move/cutout path. Helixes down from safe_z to center.z in one circular move, then does another one at final z
pub fn g2_helix(
    file: &mut dyn Write,
    center: PosRadiusAndFeed,
    safe_z: f64,
    helix_start_z: f64,
) -> Result<()> {
    let x0 = center.x.unwrap() + center.r.unwrap();
    let x1 = center.x.unwrap() - center.r.unwrap();
    let y = center.y.unwrap();
    let feed = center.feed.unwrap();
    g0(file, xyz(x0, y, safe_z))?;
    g1(file, xyzf(x0, y, helix_start_z, feed))?;
    g2(file, xyzijf(x0, y, center.z.unwrap(), x1, y, feed))?;
    g1(file, xyzf(x0, y, center.z.unwrap(), feed))?;
    g2(file, xyijf(x0, y, x1, y, feed))?;
    g1(file, zf(safe_z, feed))?;
    Ok(())
}

/// Enable inverse feed rate mode (G93)
/// With inverse feed rate mode enabled, each non-rapid move needs to contain an `F` parameter.
/// `F` is interpreted as the inverse of the feed time, in minutes. E.g. `F3.0` is interpreted
/// as "complete this move in 20 seconds"
pub fn inv_feed_g93(file: &mut dyn Write) -> Result<()> {
    writeln!(file, "G93")
}

/// Enable units-per-minute feed rate mode (G94)
pub fn standard_feed_g94(file: &mut dyn Write) -> Result<()> {
    writeln!(file, "G94")
}
