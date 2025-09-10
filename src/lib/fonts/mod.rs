use crate::{g0, g1, gcode_comment, xy, xyf, xyz, z, zf};
use roxmltree::{Document, ParsingOptions};
use std::fs::read_to_string;
use std::{collections::HashMap, io::Write, path::PathBuf};

use anyhow::{Context, Result};

pub struct Font {
    glyphs: HashMap<char, Glyph>,
    x_height: f64,
    units_per_em: f64,
}

pub struct Glyph {
    moves: Vec<Move>,
    width: f64,
}

#[derive(Debug, Clone)]
enum MoveType {
    Move,
    Line,
}

pub struct Move {
    move_type: MoveType,
    x: f64,
    y: f64,
}

impl Font {
    pub fn new_from_svg(path: &PathBuf) -> Result<Self> {
        parse_svg_xml_font(path)
    }

    pub fn string_to_gcode(
        &self,
        file: &mut dyn Write,
        s: &str,
        depth: f64,
        safe_z: f64,
        feed: f64,
        scale: f64,
    ) -> Result<()> {
        let mut x_off = 0.0;
        // For each character in the string, get the glyph and write the moves to the file
        for c in s.chars() {
            gcode_comment(file, &format!("Writing '{c}'"))?;
            let glyph = self.glyphs.get(&c).unwrap();
            // Feed to the first move, and then feed in
            g0(file, xyz(x_off, 0.0, safe_z))?;
            g1(file, zf(depth, feed))?;

            for m in &glyph.moves {
                match m.move_type {
                    MoveType::Move => {
                        // A move is a feed out, move, feed in
                        g0(file, z(safe_z))?;
                        g0(file, xy(x_off + m.x * scale, m.y * scale))?;
                        g1(file, zf(depth, feed))?;
                    }
                    MoveType::Line => {
                        // A line is a straight in-situ move
                        g1(file, xyf(x_off + m.x * scale, m.y * scale, feed))?;
                    }
                }
            }
            // Increase the x offset by the letter width
            x_off += glyph.width * scale;
            // And feed out
            g0(file, z(safe_z))?;
        }
        Ok(())
    }
}

fn parse_svg_xml_font(path: &PathBuf) -> Result<Font> {
    // Parse the svg xml path using Roxmltree
    let data = read_to_string(path)?;
    let doc = Document::parse_with_options(
        &data,
        ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )?;

    let mut glyphs = HashMap::new();
    // Get the x-height
    let x_height = doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "font-face")
        .next()
        .unwrap()
        .attribute("x-height")
        .unwrap()
        .parse::<f64>()
        .unwrap();
    // Get units per em
    let units_per_em = doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "font-face")
        .next()
        .unwrap()
        .attribute("units-per-em")
        .unwrap()
        .parse::<f64>()
        .unwrap();

    doc.descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "glyph")
        .for_each(|n| {
            // Get the unicode value, which will be the key for the font
            let name_str = n.attribute("unicode").unwrap();
            // The name is the first char of the string
            let name = name_str.chars().next().unwrap();
            // <glyph unicode="G" glyph-name="G" horiz-adv-x="624" d="M 346 315 L 520 315 L 520 81.9 L 479 53.6 L 391 22.1 L 328 18.9 L 265 31.5 L 208 63 L 142 139 L 97.6 233 L 88.2 343 L 101 450 L 142 545 L 208 617 L 274 649 L 350 662 L 432 649 L 482 621 L 517 592" />
            // Parse the 'd' attribute, turning each M into a Move and each L into a line to, in a Move
            let mut moves = Vec::new();
            let mut move_type = MoveType::Move;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;

            if let Some(d) = n.attribute("d") {
                for entry in d.trim().split(" ") {
                    if entry == "M" {
                        move_type = MoveType::Move;
                    } else if entry == "L" {
                        move_type = MoveType::Line;
                    } else if let Ok(v) = entry.parse::<f64>() {
                        // Otherwise, we parse as a float
                        if x.is_none() {
                            x = Some(v);
                        } else if y.is_none() {
                            y = Some(v);
                        } else {
                            // We have both x and y, so we can create a Move
                            moves.push(Move {
                                move_type: move_type.clone(),
                                x: x.unwrap() / units_per_em,
                                y: y.unwrap() / units_per_em,
                            });
                            x = None;
                            y = None;
                        }
                    }
                }
            }
            // Example entry

            glyphs.insert(
                name,
                Glyph {
                    moves,
                    width: n.attribute("horiz-adv-x").unwrap().parse::<f64>().unwrap()
                        / units_per_em,
                },
            );
        });

    Ok(Font {
        glyphs,
        x_height,
        units_per_em,
    })
}
