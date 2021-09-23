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
