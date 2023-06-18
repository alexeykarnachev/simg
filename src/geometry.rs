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

pub fn get_circle_polygon_mtv(
    circle: &Circle,
    vertices: &[Vector2<f32>],
) -> Option<Vector2<f32>> {
    let mut nearest_vertex = Vector2::zeros();
    let mut min_overlap_axis = Vector2::zeros();
    let mut min_overlap = f32::INFINITY;
    let mut nearest_dist = f32::INFINITY;
    for i in 0..vertices.len() {
        let v0 = vertices[i];
        let v1 = if i < vertices.len() - 1 {
            vertices[i + 1]
        } else {
            vertices[0]
        };
        let axis = rotate90(&(v1 - v0)).normalize();
        let bound0 = get_circle_proj_bound(circle, &axis);
        let bound1 = get_polygon_proj_bound(vertices, &axis);

        update_overlap(
            &bound0,
            &bound1,
            &axis,
            &mut min_overlap_axis,
            &mut min_overlap,
        );

        let curr_dist = v0.metric_distance(&circle.center);
        if curr_dist < nearest_dist {
            nearest_dist = curr_dist;
            nearest_vertex = v0;
        }
    }

    let axis = (circle.center - nearest_vertex).normalize();
    let bound0 = get_circle_proj_bound(circle, &axis);
    let bound1 = get_polygon_proj_bound(vertices, &axis);
    update_overlap(
        &bound0,
        &bound1,
        &axis,
        &mut min_overlap_axis,
        &mut min_overlap,
    );

    if min_overlap > 0.0 {
        return Some(-min_overlap * min_overlap_axis);
    }

    return None;
}

pub fn get_rectangle_rectangle_mtv(
    rect0: &Rectangle,
    rect1: &Rectangle,
) -> Option<Vector2<f32>> {
    let mut mtv = Vector2::zeros();

    let sum_width = rect0.get_width() + rect1.get_width();
    let sum_height = rect0.get_height() + rect1.get_height();

    let min_x = rect0.get_left_x().min(rect1.get_left_x());
    let max_x = rect0.get_right_x().max(rect1.get_right_x());
    let min_y = rect0.get_bot_y().min(rect1.get_bot_y());
    let max_y = rect0.get_top_y().max(rect1.get_top_y());

    let width = max_x - min_x;
    let height = max_y - min_y;

    if width <= sum_width && height <= sum_height {
        mtv.x = if rect1.get_right_x() > rect0.get_right_x() {
            rect1.get_left_x() - rect0.get_right_x()
        } else {
            rect1.get_right_x() - rect0.get_left_x()
        };

        mtv.y = if rect1.get_top_y() > rect0.get_top_y() {
            rect1.get_bot_y() - rect0.get_top_y()
        } else {
            rect1.get_top_y() - rect0.get_bot_y()
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
    get_circle_polygon_mtv(circle, &rect.get_vertices())
}

fn get_circle_proj_bound(
    circle: &Circle,
    axis: &Vector2<f32>,
) -> Vector2<f32> {
    let axis = axis.normalize();
    let r = axis * circle.radius;
    let k0 = axis.dot(&(circle.center - r));
    let k1 = axis.dot(&(circle.center + r));

    Vector2::new(k0, k1)
}

fn get_polygon_proj_bound(
    vertices: &[Vector2<f32>],
    axis: &Vector2<f32>,
) -> Vector2<f32> {
    let axis = axis.normalize();
    let mut min_k = f32::INFINITY;
    let mut max_k = -f32::INFINITY;
    for i in 0..vertices.len() {
        let k = axis.dot(&vertices[i]);
        min_k = min_k.min(k);
        max_k = max_k.max(k);
    }

    Vector2::new(min_k, max_k)
}

fn update_overlap(
    bound0: &Vector2<f32>,
    bound1: &Vector2<f32>,
    axis: &Vector2<f32>,
    min_overlap_axis: &mut Vector2<f32>,
    min_overlap: &mut f32,
) {
    let r0 = 0.5 * (bound0.y - bound0.x);
    let r1 = 0.5 * (bound1.y - bound1.x);
    let c0 = 0.5 * (bound0.y + bound0.x);
    let c1 = 0.5 * (bound1.y + bound1.x);
    let radii_sum = r0 + r1;
    let dist = (c1 - c0).abs();
    let overlap = radii_sum - dist;
    if overlap < *min_overlap {
        *min_overlap = overlap;
        *min_overlap_axis = if c1 - c0 < 0.0 { -axis } else { *axis }
    }
}
