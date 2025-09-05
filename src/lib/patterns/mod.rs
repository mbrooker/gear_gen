use core::f64;
use std::io::Result;
use std::io::Write;

use crate::g3;
use crate::{g0, g1, g2, xy, xyf, xyijf, z, zf, PosAndFeed};

pub fn radial_tick_marks(
    file: &mut dyn Write,
    inner_rad: f64,
    outer_rad: f64,
    n: usize,
    center: &PosAndFeed,
    z_cut: f64,
    skips_mods: &[usize],
) -> Result<()> {
    let z_safe = 1.0;
    let cx = center.x.unwrap();
    let cy = center.y.unwrap();
    // Double check we're at a safe Z
    g0(file, z(z_safe))?;
    // Now draw the radial ticks
    'tick_loop: for i in 0..n {
        for modulus in skips_mods {
            if i % modulus == 0 {
                continue 'tick_loop;
            }
        }
        let angle = i as f64 * f64::consts::TAU / n as f64;
        g0(
            file,
            xy(inner_rad * angle.sin() + cx, inner_rad * angle.cos() + cy),
        )?;
        g1(file, zf(z_cut, center.feed.unwrap()))?;
        g1(
            file,
            xyf(
                outer_rad * angle.sin() + cx,
                outer_rad * angle.cos() + cy,
                center.feed.unwrap(),
            ),
        )?;
        g1(file, zf(z_safe, center.feed.unwrap()))?;
    }
    Ok(())
}

pub fn radial_tick_segments(
    file: &mut dyn Write,
    inner_rad: f64,
    outer_rad: f64,
    n: usize,
    center: &PosAndFeed,
    z_cut: f64,
    inc_angle: f64,
) -> Result<()> {
    let z_safe = 1.0;
    let cx = center.x.unwrap();
    let cy = center.y.unwrap();
    let feed = center.feed.unwrap();
    // Double check we're at a safe Z
    g0(file, z(z_safe))?;
    // Now draw the radial ticks
    for i in 0..n {
        let base_angle = i as f64 * f64::consts::TAU / n as f64;

        let left_angle = base_angle - inc_angle / 2.0;

        let sx1 = inner_rad * left_angle.sin() + cx;
        let sy1 = inner_rad * left_angle.cos() + cy;

        g0(file, xy(sx1, sy1))?;
        g1(file, zf(z_cut, feed))?;

        let ex1 = outer_rad * left_angle.sin() + cx;
        let ey1 = outer_rad * left_angle.cos() + cy;

        g1(file, xyf(ex1, ey1, feed))?;

        let right_angle = left_angle + inc_angle;
        let sx2 = outer_rad * right_angle.sin() + cx;
        let sy2 = outer_rad * right_angle.cos() + cy;
        // Outer arc segment
        g2(file, xyijf(sx2, sy2, -ex1, -ey1, feed))?;

        let ex2 = inner_rad * right_angle.sin() + cx;
        let ey2 = inner_rad * right_angle.cos() + cy;
        g1(file, xyf(ex2, ey2, feed))?;

        // Inner arc segment
        g3(file, xyijf(sx1, sy1, -ex2, -ey2, feed))?;

        // Raise the cutter, ready for the next rapid
        g1(file, zf(z_safe, center.feed.unwrap()))?;
    }
    Ok(())
}
