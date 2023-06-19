use crate::shapes::*;
use nalgebra::Vector2;

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

pub fn check_if_point_in_rectangle(
    point: &Vector2<f32>,
    rect: &Rectangle,
) -> bool {
    point.x > rect.get_min_x()
        && point.x < rect.get_max_x()
        && point.y > rect.get_min_y()
        && point.y < rect.get_max_y()
}
