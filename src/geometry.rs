use crate::shapes::*;
use core::f32::consts::PI;
use nalgebra::Vector2;
use rand::Rng;

pub const UP: Vector2<f32> = Vector2::new(0.0, 1.0);
pub const DOWN: Vector2<f32> = Vector2::new(0.0, -1.0);
pub const LEFT: Vector2<f32> = Vector2::new(-1.0, 0.0);
pub const RIGHT: Vector2<f32> = Vector2::new(1.0, 0.0);

pub fn rotate90(vec: &Vector2<f32>) -> Vector2<f32> {
    Vector2::new(-vec.y, vec.x)
}

pub fn reflect(vec: &Vector2<f32>, normal: &Vector2<f32>) -> Vector2<f32> {
    let normal = normal.normalize();

    vec - 2.0 * vec.dot(&normal) * normal
}

pub fn get_unit_2d_by_random() -> Vector2<f32> {
    let mut rng = rand::thread_rng();
    let angle = rng.gen_range(-PI..=PI);

    get_unit_2d_by_angle(angle)
}

pub fn get_unit_2d_by_angle(angle: f32) -> Vector2<f32> {
    Vector2::new(angle.cos(), angle.sin())
}

pub fn get_rectangle_rectangle_mtv(
    rect0: &Rectangle,
    rect1: &Rectangle,
) -> Option<Vector2<f32>> {
    let mut mtv = Vector2::zeros();

    let sum_width = rect0.get_width() + rect1.get_width();
    let sum_height = rect0.get_height() + rect1.get_height();

    let min_x = rect0.get_min_x().min(rect1.get_min_x());
    let max_x = rect0.get_max_x().max(rect1.get_max_x());
    let min_y = rect0.get_min_y().min(rect1.get_min_y());
    let max_y = rect0.get_max_y().max(rect1.get_max_y());

    let width = max_x - min_x;
    let height = max_y - min_y;

    if width <= sum_width && height <= sum_height {
        mtv.x = if rect1.get_max_x() > rect0.get_max_x() {
            rect1.get_min_x() - rect0.get_max_x()
        } else {
            rect1.get_max_x() - rect0.get_min_x()
        };

        mtv.y = if rect1.get_max_y() > rect0.get_max_y() {
            rect1.get_min_y() - rect0.get_max_y()
        } else {
            rect1.get_max_y() - rect0.get_min_y()
        };

        if mtv.x.abs() > mtv.y.abs() {
            mtv.x = 0.0;
        } else {
            mtv.y = 0.0;
        }

        return Some(mtv);
    }

    None
}

pub fn get_circle_circle_mtv(
    circle0: &Circle,
    circle1: &Circle,
) -> Option<Vector2<f32>> {
    let diff = circle1.center - circle0.center;
    let dist = diff.magnitude_squared().sqrt();
    let axis = if dist > f32::EPSILON {
        diff / dist
    } else {
        RIGHT
    };

    let radii_sum = circle0.radius + circle1.radius;
    if dist < radii_sum {
        return Some(axis * (dist - radii_sum));
    }

    None
}

pub fn get_circle_rectangle_mtv(
    circle: &Circle,
    rect: &Rectangle,
) -> Option<Vector2<f32>> {
    let r = circle.radius;
    let c = circle.center;
    let c_min_x = circle.get_min_x();
    let c_max_x = circle.get_max_x();
    let c_max_y = circle.get_max_y();
    let c_min_y = circle.get_min_y();
    let r_min_x = rect.get_min_x();
    let r_max_x = rect.get_max_x();
    let r_max_y = rect.get_max_y();
    let r_min_y = rect.get_min_y();

    // Center of the cicle is fully inside the rectangle
    if check_if_point_in_rectangle(&circle.center, rect) {
        let left = c.x - r_min_x;
        let right = r_max_x - c.x;
        let top = r_max_y - c.y;
        let bot = c.y - r_min_y;
        let min = *[left, right, top, bot]
            .iter()
            .min_by(|&a, &b| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        if min == left {
            return Some(Vector2::new(-left - r, 0.0));
        } else if min == right {
            return Some(Vector2::new(right + r, 0.0));
        } else if min == top {
            return Some(Vector2::new(0.0, top + r));
        } else if min == bot {
            return Some(Vector2::new(0.0, -bot - r));
        }
    // One of the circle "corners" is fully inside the rectangle
    } else if check_if_point_in_rectangle(&circle.get_left(), rect) {
        return Some(Vector2::new(r_max_x - c_min_x, 0.0));
    } else if check_if_point_in_rectangle(&circle.get_right(), rect) {
        return Some(Vector2::new(r_min_x - c_max_x, 0.0));
    } else if check_if_point_in_rectangle(&circle.get_top(), rect) {
        return Some(Vector2::new(0.0, r_min_y - c_max_y));
    } else if check_if_point_in_rectangle(&circle.get_bot(), rect) {
        return Some(Vector2::new(0.0, r_max_y - c_min_y));
    // One of the rectangle corners is iside the circle
    } else {
        let r_tl = rect.get_top_left();
        let r_tr = rect.get_top_right();
        let r_bl = rect.get_bot_left();
        let r_br = rect.get_bot_right();
        let tl = c.metric_distance(&r_tl);
        let tr = c.metric_distance(&r_tr);
        let bl = c.metric_distance(&r_bl);
        let br = c.metric_distance(&r_br);
        let min = *[tl, tr, bl, br]
            .iter()
            .min_by(|&a, &b| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        if min >= r {
            return None;
        } else if min == tl {
            return Some((c - r_tl).normalize() * (r - tl));
        } else if min == tr {
            return Some((c - r_tr).normalize() * (r - tr));
        } else if min == bl {
            return Some((c - r_bl).normalize() * (r - bl));
        } else if min == br {
            return Some((c - r_br).normalize() * (r - br));
        }
    }

    None
}

pub fn intersect_line_with_circle(
    line: &Line,
    circle: &Circle,
) -> [Option<Vector2<f32>>; 2] {
    let s = line.s;
    let e = line.e;
    let r = circle.radius;

    let x1 = s.x;
    let y1 = s.y;
    let x2 = e.x;
    let y2 = e.y;
    let cx = circle.center.x;
    let cy = circle.center.y;

    let dx = x2 - x1;
    let dy = y2 - y1;
    let a = dx * dx + dy * dy;
    let b = 2.0 * (dx * (x1 - cx) + dy * (y1 - cy));
    let c = cx * cx + cy * cy + x1 * x1 + y1 * y1
        - 2.0 * (cx * x1 + cy * y1)
        - r * r;

    let det = b * b - 4.0 * a * c;
    let mut t1 = 2.0;
    let mut t2 = 2.0;
    if det == 0.0 {
        t1 = -b / (2.0 * a);
    } else if det > 0.0 {
        let det_sqrt = det.sqrt();
        t1 = (-b + det_sqrt) / (2.0 * a);
        t2 = (-b - det_sqrt) / (2.0 * a);
    }

    let mut point0 = None;
    if t1 >= 0.0 && t1 <= 1.0 {
        point0 = Some(Vector2::new(x1 + t1 * dx, y1 + t1 * dy));
    }

    let mut point1 = None;
    if t2 >= 0.0 && t2 <= 1.0 {
        point1 = Some(Vector2::new(x1 + t2 * dx, y1 + t2 * dy));
    }

    let mut points = [None, None];
    if point0.is_some() && point1.is_none() {
        points[0] = point0;
    } else if point0.is_none() && point1.is_some() {
        points[0] = point1;
    } else if let (Some(point0), Some(point1)) = (point0, point1) {
        if point0.metric_distance(&s) <= point1.metric_distance(&s) {
            points[0] = Some(point0);
            points[1] = Some(point1);
        } else {
            points[0] = Some(point1);
            points[1] = Some(point0);
        }
    }

    return points;
}

pub fn check_if_point_in_rectangle(
    point: &Vector2<f32>,
    rect: &Rectangle,
) -> bool {
    point.x > rect.get_min_x()
        && point.x < rect.get_max_x()
        && point.y > rect.get_min_y()
        && point.y < rect.get_max_y()
}
