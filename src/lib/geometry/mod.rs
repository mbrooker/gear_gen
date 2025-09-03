use nalgebra::geometry::Point2;

#[derive(Debug)]
pub struct LineSegment {
    pub start: Point2<f64>,
    pub end: Point2<f64>,
}

#[derive(Debug)]
pub struct Circle {
    pub center: Point2<f64>,
    pub radius: f64,
}

/// Given the LineSegment `line`, return zero or one lines corresponding to the portion of the segment that is inside the circle `c`
/// If `line` is tangent to `circle`, return None instead of a zero-length line
pub fn trim(line: LineSegment, circle: Circle) -> Option<LineSegment> {
    // Vector from circle center to line start
    let to_start = line.start - circle.center;

    // Line direction vector
    let dir = line.end - line.start;
    let len_sq = dir.norm_squared();

    // Handle degenerate case (zero-length line)
    if len_sq == 0.0 {
        return if to_start.norm_squared() <= circle.radius * circle.radius {
            Some(line)
        } else {
            None
        };
    }

    // Solve quadratic equation for line-circle intersection
    // Line: P(t) = start + t * dir, where t ∈ [0, 1]
    // Circle: |P(t) - center|² = radius²
    let a = len_sq;
    let b = 2.0 * to_start.dot(&dir);
    let c = to_start.norm_squared() - circle.radius * circle.radius;

    let discriminant = b * b - 4.0 * a * c;

    println!("Got discriminant {discriminant}: {a} {b} {c}");
    // No intersection, or tangent
    if discriminant <= 0.0 {
        return None;
    }

    let sqrt_disc = discriminant.sqrt();
    let t1 = (-b - sqrt_disc) / (2.0 * a);
    let t2 = (-b + sqrt_disc) / (2.0 * a);

    if ((t1 > 1.0) && (t2 > 1.0)) || ((t1 < 0.0) && (t2 < 0.0)) {
        return None;
    }

    // Clamp intersection parameters to [0, 1] (segment bounds)
    let t_min = t1.max(0.0).min(1.0);
    let t_max = t2.max(0.0).min(1.0);

    println!("{t1} {t2} {t_min} {t_max}");

    // Calculate intersection points
    let p1 = line.start + t_min * dir;
    let p2 = line.start + t_max * dir;

    // Return the trimmed segment
    Some(LineSegment { start: p1, end: p2 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::geometry::Point2;

    const EPSILON: f64 = 1e-10;

    fn points_equal(p1: Point2<f64>, p2: Point2<f64>) -> bool {
        (p1 - p2).norm() < EPSILON
    }

    #[test]
    fn test_line_completely_outside_circle() {
        let circle = Circle {
            center: Point2::new(0.0, 0.0),
            radius: 1.0,
        };
        let line = LineSegment {
            start: Point2::new(2.0, 0.0),
            end: Point2::new(3.0, 0.0),
        };

        let result = trim(line, circle);
        println!("Got outside result {:?}", result);
        assert!(result.is_none());
    }

    #[test]
    fn test_line_completely_outside_circle_miss() {
        let circle = Circle {
            center: Point2::new(0.0, 0.0),
            radius: 1.0,
        };
        let line = LineSegment {
            start: Point2::new(0.0, 2.0),
            end: Point2::new(2.0, 2.0),
        };

        let result = trim(line, circle);
        println!("Got outside result {:?}", result);
        assert!(result.is_none());
    }

    #[test]
    fn test_line_completely_inside_circle() {
        let circle = Circle {
            center: Point2::new(0.0, 0.0),
            radius: 2.0,
        };
        let line = LineSegment {
            start: Point2::new(-0.5, 0.0),
            end: Point2::new(0.5, 0.0),
        };

        let result = trim(line, circle).unwrap();
        assert!(points_equal(result.start, Point2::new(-0.5, 0.0)));
        assert!(points_equal(result.end, Point2::new(0.5, 0.0)));
    }

    #[test]
    fn test_line_both_ends_outside_passes_through() {
        let circle = Circle {
            center: Point2::new(0.0, 0.0),
            radius: 1.0,
        };
        let line = LineSegment {
            start: Point2::new(-2.0, 0.0),
            end: Point2::new(2.0, 0.0),
        };

        let result = trim(line, circle).unwrap();
        assert!(points_equal(result.start, Point2::new(-1.0, 0.0)));
        assert!(points_equal(result.end, Point2::new(1.0, 0.0)));
    }

    #[test]
    fn test_line_one_end_inside_one_outside() {
        let circle = Circle {
            center: Point2::new(0.0, 0.0),
            radius: 1.0,
        };
        let line = LineSegment {
            start: Point2::new(0.5, 0.0), // Inside (at center)
            end: Point2::new(2.0, 0.0),   // Outside
        };

        let result = trim(line, circle).unwrap();
        assert!(points_equal(result.start, Point2::new(0.5, 0.0)));
        assert!(points_equal(result.end, Point2::new(1.0, 0.0)));
    }

    #[test]
    fn test_line_one_end_outside_one_inside() {
        let circle = Circle {
            center: Point2::new(0.0, 0.0),
            radius: 1.0,
        };
        let line = LineSegment {
            start: Point2::new(-2.0, 0.0), // Outside
            end: Point2::new(0.5, 0.0),    // Inside
        };

        let result = trim(line, circle).unwrap();
        assert!(points_equal(result.start, Point2::new(-1.0, 0.0)));
        assert!(points_equal(result.end, Point2::new(0.5, 0.0)));
    }

    #[test]
    fn test_line_tangent_to_circle() {
        let circle = Circle {
            center: Point2::new(0.0, 0.0),
            radius: 1.0,
        };
        let line = LineSegment {
            start: Point2::new(-1.0, 1.0), // Tangent line at y=1
            end: Point2::new(1.0, 1.0),
        };

        let result = trim(line, circle).unwrap();
        // For tangent case, both points should be at the tangent point
        assert!(points_equal(result.start, Point2::new(0.0, 1.0)));
        assert!(points_equal(result.end, Point2::new(0.0, 1.0)));
    }

    #[test]
    fn test_zero_length_line_inside_circle() {
        let circle = Circle {
            center: Point2::new(0.0, 0.0),
            radius: 1.0,
        };
        let line = LineSegment {
            start: Point2::new(0.5, 0.0),
            end: Point2::new(0.5, 0.0),
        };

        let result = trim(line, circle).unwrap();
        assert!(points_equal(result.start, Point2::new(0.5, 0.0)));
        assert!(points_equal(result.end, Point2::new(0.5, 0.0)));
    }

    #[test]
    fn test_zero_length_line_outside_circle() {
        let circle = Circle {
            center: Point2::new(0.0, 0.0),
            radius: 1.0,
        };
        let line = LineSegment {
            start: Point2::new(2.0, 0.0),
            end: Point2::new(2.0, 0.0),
        };

        let result = trim(line, circle);
        assert!(result.is_none());
    }

    #[test]
    fn test_diagonal_line_intersection() {
        let circle = Circle {
            center: Point2::new(0.0, 0.0),
            radius: 1.0,
        };
        let line = LineSegment {
            start: Point2::new(-2.0, -2.0),
            end: Point2::new(2.0, 2.0),
        };

        let result = trim(line, circle).unwrap();
        // For a diagonal line y=x through a unit circle, intersection points are at (±√2/2, ±√2/2)
        let expected_coord = 1.0 / 2.0_f64.sqrt();
        assert!(points_equal(
            result.start,
            Point2::new(-expected_coord, -expected_coord)
        ));
        assert!(points_equal(
            result.end,
            Point2::new(expected_coord, expected_coord)
        ));
    }

    #[test]
    fn test_line_misses_circle_parallel() {
        let circle = Circle {
            center: Point2::new(0.0, 0.0),
            radius: 1.0,
        };
        let line = LineSegment {
            start: Point2::new(-1.0, 2.0), // Parallel to x-axis, above circle
            end: Point2::new(1.0, 2.0),
        };

        let result = trim(line, circle);
        assert!(result.is_none());
    }
}
