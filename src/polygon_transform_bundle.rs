use bevy::math::Vec2;
use bevy::prelude::Transform;
use bevy_rapier2d::na::Point2;
use bevy_rapier2d::parry::shape::SegmentPointLocation;
use bevy_rapier2d::parry::utils::segments_intersection2d;
use bevy_rapier2d::parry::utils::SegmentsIntersection::Point;
use crate::polygon::Polygon;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PolygonTransformBundle {
    pub(crate) polygon: Polygon,
    transform: Transform,
}

impl From<(Polygon, Transform)> for PolygonTransformBundle {
    fn from((polygon, transform): (Polygon, Transform)) -> Self {
        Self { polygon, transform }
    }
}

impl PolygonTransformBundle {
    pub(crate) fn sink(self, bounds: &PolygonTransformBundle) -> Self {
        let vertices = self.polygon.to_global_space(&self.transform).vertices;
        let bounds_vertices = bounds.polygon.to_global_space(&bounds.transform).vertices;

        let mut new_vertices = vec![];
        let mut is_tracing_self = true;
        let mut start_index = 0;
        let mut end_index = 1;
        let mut start_bounds_index = 1;
        let mut end_bounds_index = 0;
        let mut intersection = None;
        for _ in 0..bounds_vertices.len() {
            let start_bounds = bounds_vertices[start_bounds_index];
            let end_bounds = bounds_vertices[end_bounds_index];
            intersection = my_intersection(vertices[0], vertices[1], start_bounds, end_bounds)
                .filter(|_| cross(vertices[0], vertices[1], start_bounds) < 0.);
            if intersection.is_some() { break; }

            start_bounds_index = end_bounds_index;
            end_bounds_index = (end_bounds_index + bounds_vertices.len() - 1) % bounds_vertices.len();
        }

        while new_vertices.is_empty() || start_index != 0 || !is_tracing_self {
            if is_tracing_self {
                let start = intersection.unwrap_or(vertices[start_index]);
                let end = vertices[end_index];
                new_vertices.push(start);
                for _ in 0..bounds_vertices.len() {
                    let start_bounds = bounds_vertices[start_bounds_index];
                    let end_bounds = bounds_vertices[end_bounds_index];
                    intersection = my_intersection(start, end, start_bounds, end_bounds)
                        .filter(|_| cross(start, end, start_bounds) > 0.);
                    if intersection.is_some() { break; }

                    start_bounds_index = end_bounds_index;
                    end_bounds_index = (end_bounds_index + bounds_vertices.len() - 1) % bounds_vertices.len();
                }
            } else {
                let start_bounds = intersection.unwrap_or(bounds_vertices[start_bounds_index]);
                new_vertices.push(start_bounds);
                for _ in 0..vertices.len() {
                    intersection = my_intersection(
                        start_bounds,
                        bounds_vertices[end_bounds_index],
                        vertices[start_index],
                        vertices[end_index],
                    );
                    if intersection.is_some() { break; }

                    start_index = end_index;
                    end_index = (end_index + 1) % vertices.len();
                }
            }

            if intersection.is_some() {
                is_tracing_self = !is_tracing_self;
            } else if is_tracing_self {
                start_index = end_index;
                end_index = (end_index + 1) % vertices.len();
            } else {
                start_bounds_index = end_bounds_index;
                end_bounds_index = (end_bounds_index + bounds_vertices.len() - 1) % bounds_vertices.len();
            }
        }

        return PolygonTransformBundle {
            polygon: Polygon::from(new_vertices).to_local_space(self.transform),
            transform: self.transform,
        };
    }
}

fn my_intersection(a_start: Vec2, a_end: Vec2, b_start: Vec2, b_end: Vec2) -> Option<Vec2> {
    match segments_intersection2d(
        &Point2::from(a_start),
        &Point2::from(a_end),
        &Point2::from(b_start),
        &Point2::from(b_end),
        0.000001,
    ) {
        Some(Point { loc1: SegmentPointLocation::OnEdge([_, from_start]), .. }) =>
            Some(a_start + from_start * (a_end - a_start)),
        _ => None
    }
}

fn cross(a_start: Vec2, a_end: Vec2, b_start: Vec2) -> f32 {
    (a_end.x - a_start.x) * (b_start.y - a_start.y) - (a_end.y - a_start.y) * (b_start.x - a_start.x)
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;
    use std::io;
    use bevy::math::Vec2;
    use bevy::prelude::Transform;
    use svg::Document;
    use svg::node::element::Path;
    use svg::node::element::path::Data;
    use crate::polygon::{Polygon};
    use crate::polygon_transform_bundle::PolygonTransformBundle;

    fn svg_path(bundle: &PolygonTransformBundle, stroke: &str, stroke_width: f64) -> Path {
        return Path::new()
            .set("fill", "none")
            .set("stroke", stroke)
            .set("stroke-width", stroke_width)
            .set("d", svg_path_data(bundle));
    }

    fn svg_path_data(bundle: &PolygonTransformBundle) -> Data {
        let vertices = bundle.polygon.to_global_space(&bundle.transform).vertices;

        let mut data = Data::new()
            .move_to((vertices[0].x, -vertices[0].y));

        for vertex in vertices.iter().skip(1) {
            data = data.line_to((vertex.x, -vertex.y));
        }

        return data.close();
    }

    fn save_svg(document: Document, stable_name: &str) -> io::Result<String> {
        let comparison_image = &format!("target/{stable_name}.svg");
        svg::save(comparison_image, &document).unwrap();

        let parent = current_dir()?;
        return Ok(format!("file:///{}", parent.join(comparison_image).display()));
    }

    #[test]
    fn test_sink_intersect_start() {
        let left_operand = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(2., 2.),
                Vec2::new(2., -2.),
                Vec2::new(-2., -2.),
                Vec2::new(-2., 2.),
            ]),
            transform: Transform::from_xyz(0., 0., 0.),
        };

        let right_operand = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(3., 3.),
                Vec2::new(3., 1.),
                Vec2::new(1., 1.),
                Vec2::new(1., 3.),
            ]),
            transform: Transform::from_xyz(0., 0., 0.),
        };

        let actual = left_operand.clone().sink(&right_operand);
        let expected = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(2., 1.),
                Vec2::new(2., -2.),
                Vec2::new(-2., -2.),
                Vec2::new(-2., 2.),
                Vec2::new(1., 2.),
                Vec2::new(1., 1.),
            ]),
            transform: Transform::from_xyz(0., 0., 0.),
        };

        let scene = Document::new()
            .set("viewBox", (-3, -3, 6, 6))
            .add(svg_path(&actual, "red", 0.25))
            .add(svg_path(&expected, "green", 0.125))
            .add(svg_path(&left_operand, "black", 0.125 / 4.))
            .add(svg_path(&right_operand, "white", 0.125 / 4.))
            ;

        assert_eq!(actual, expected, "Visual: {:?}", save_svg(scene, "test_sink_intersect_start"))
    }

    #[test]
    fn test_sink_simple_subtract() {
        let left_operand = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(2., 2.),
                Vec2::new(2., -2.),
                Vec2::new(-2., -2.),
                Vec2::new(-2., 2.),
            ]),
            transform: Transform::from_xyz(0., 0., 0.),
        };

        let right_operand = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(1., 3.),
                Vec2::new(1., 1.),
                Vec2::new(-1., 1.),
                Vec2::new(-1., 3.),
            ]),
            transform: Transform::from_xyz(0., 0., 0.),
        };

        let actual = left_operand.clone().sink(&right_operand);
        let expected = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(2., 2.),
                Vec2::new(2., -2.),
                Vec2::new(-2., -2.),
                Vec2::new(-2., 2.),
                Vec2::new(-1., 2.),
                Vec2::new(-1., 1.),
                Vec2::new(1., 1.),
                Vec2::new(1., 2.),
            ]),
            transform: Transform::from_xyz(0., 0., 0.),
        };

        let scene = Document::new()
            .set("viewBox", (-3, -3, 6, 6))
            .add(svg_path(&actual, "red", 0.25))
            .add(svg_path(&expected, "green", 0.125))
            .add(svg_path(&left_operand, "black", 0.125 / 4.))
            .add(svg_path(&right_operand, "white", 0.125 / 4.))
            ;

        assert_eq!(actual, expected, "Visual: {:?}", save_svg(scene, "test_sink_simple_subtract"))
    }

    #[test]
    fn test_sink_double_subtract() {
        let left_operand = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(2., 2.),
                Vec2::new(2., 0.),
                Vec2::new(-3., 0.),
                Vec2::new(-3., 2.),
            ]),
            transform: Transform::from_xyz(1., 0., 0.),
        };

        let right_operand = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(1., 4.),
                Vec2::new(1., 1.),
                Vec2::new(0., 1.),
                Vec2::new(0., 3.),
                Vec2::new(-1., 3.),
                Vec2::new(-1., 1.),
                Vec2::new(-2., 1.),
                Vec2::new(-2., 4.),
            ]),
            transform: Transform::from_xyz(1., 0., 0.),
        };

        let actual = left_operand.clone().sink(&right_operand);
        let expected = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(2., 2.),
                Vec2::new(2., 0.),
                Vec2::new(-3., 0.),
                Vec2::new(-3., 2.),
                Vec2::new(-2., 2.),
                Vec2::new(-2., 1.),
                Vec2::new(-1., 1.),
                Vec2::new(-1., 2.),
                Vec2::new(0., 2.),
                Vec2::new(0., 1.),
                Vec2::new(1., 1.),
                Vec2::new(1., 2.),
            ]),
            transform: Transform::from_xyz(1., 0., 0.),
        };


        let scene = Document::new()
            .set("viewBox", (-3, -4, 7, 6))
            .add(svg_path(&actual, "red", 0.25))
            .add(svg_path(&expected, "green", 0.125))
            .add(svg_path(&left_operand, "black", 0.125 / 4.))
            .add(svg_path(&right_operand, "white", 0.125 / 4.))
            ;

        assert_eq!(actual, expected, "Visual: {:?}", save_svg(scene, "test_sink_double_subtract"))
    }
}