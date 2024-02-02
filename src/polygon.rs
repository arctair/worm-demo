use bevy::math::Vec2;
use bevy::prelude::{Component, Transform};
use bevy_rapier2d::na::Point2;
use bevy_rapier2d::parry::shape::SegmentPointLocation;
use bevy_rapier2d::parry::utils::segments_intersection2d;
use bevy_rapier2d::parry::utils::SegmentsIntersection::Point;
use svg::node::element::Path;
use svg::node::element::path::Data;

#[derive(Clone, Component, Debug, PartialEq)]
pub(crate) struct Polygon {
    pub(crate) vertices: Vec<Vec2>,
}

impl From<Vec<Vec2>> for Polygon {
    fn from(vertices: Vec<Vec2>) -> Self {
        Polygon { vertices }
    }
}

impl Polygon {
    pub(crate) fn to_global_space(&self, transform: &Transform) -> Polygon {
        let mut global_vertices = self.vertices.clone();
        for index in 0..global_vertices.len() {
            global_vertices[index] *= transform.scale.truncate();
            global_vertices[index] += transform.translation.truncate();
        }
        return Polygon::from(global_vertices);
    }

    fn to_local_space(&self, transform: Transform) -> Polygon {
        let mut local_vertices = self.vertices.clone();
        for index in 0..local_vertices.len() {
            local_vertices[index] -= transform.translation.truncate();
            local_vertices[index] /= transform.scale.truncate();
        }
        return Polygon::from(local_vertices);
    }
}

#[derive(Clone, Copy, Debug)]
enum TraceMode {
    TracingSelf,
    TracingBounds,
}

#[derive(Clone, Debug, PartialEq)]
struct PolygonTransformBundle {
    polygon: Polygon,
    transform: Transform,
}

impl PolygonTransformBundle {
    fn sink(self, area: f32, bounds: Self) -> Self {
        let vertices = self.polygon.to_global_space(&self.transform).vertices;
        let bounds_vertices = bounds.polygon.to_global_space(&bounds.transform).vertices;

        let mut new_vertices = vec![];
        let mut trace_mode = TraceMode::TracingSelf;
        let mut start_index = 0;
        let mut end_index = 1;
        let mut start_bounds_index = 1;
        let mut end_bounds_index = 0;
        let mut intersection = None;

        while new_vertices.is_empty() || start_index != 0 || match trace_mode {
            TraceMode::TracingBounds => true,
            _ => false
        } {
            match trace_mode {
                TraceMode::TracingSelf => {
                    let start = intersection.unwrap_or(vertices[start_index]);
                    let end = vertices[end_index];
                    new_vertices.push(start);
                    for _ in 0..bounds_vertices.len() {
                        let start_bounds = bounds_vertices[start_bounds_index];
                        let end_bounds = bounds_vertices[end_bounds_index];
                        intersection = intersection_contains(start, end, start_bounds, end_bounds);
                        if intersection.is_some() { break; }

                        start_bounds_index = end_bounds_index;
                        end_bounds_index = (end_bounds_index + bounds_vertices.len() - 1) % bounds_vertices.len();
                    }
                }
                TraceMode::TracingBounds => {
                    let start_bounds = intersection.unwrap_or(bounds_vertices[start_bounds_index]);
                    new_vertices.push(start_bounds);
                    for _ in 0..vertices.len() {
                        intersection = intersection_contains(
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
            }

            match (trace_mode, intersection) {
                (TraceMode::TracingSelf, Some(_)) => trace_mode = TraceMode::TracingBounds,
                (TraceMode::TracingBounds, Some(_)) => trace_mode = TraceMode::TracingSelf,
                (TraceMode::TracingSelf, None) => {
                    start_index = end_index;
                    end_index = (end_index + 1) % vertices.len();
                }
                (TraceMode::TracingBounds, None) => {
                    start_bounds_index = end_bounds_index;
                    end_bounds_index = (end_bounds_index + bounds_vertices.len() - 1) % bounds_vertices.len();
                }
            }
        }

        return PolygonTransformBundle {
            polygon: Polygon::from(new_vertices).to_local_space(self.transform),
            transform: self.transform,
        };
    }

    pub(crate) fn svg_path(&self, stroke: &str, stroke_width: f64) -> Path {
        return Path::new()
            .set("fill", "none")
            .set("stroke", stroke)
            .set("stroke-width", stroke_width)
            .set("d", self.svg_path_data());
    }

    fn svg_path_data(&self) -> Data {
        let vertices = self.polygon.to_global_space(&self.transform).vertices;

        let mut data = Data::new()
            .move_to((vertices[0].x, -vertices[0].y));

        for vertex in vertices.iter().skip(1) {
            data = data.line_to((vertex.x, -vertex.y));
        }

        return data.close();
    }
}

fn intersection_contains(a_start: Vec2, a_end: Vec2, b_start: Vec2, b_end: Vec2) -> Option<Vec2> {
    if (a_end.x - a_start.x) * (b_start.y - a_start.y) - (a_end.y - a_start.y) * (b_start.x - a_start.x) > 0. {
        match segments_intersection2d(
            &Point2::from(a_start),
            &Point2::from(a_end),
            &Point2::from(b_start),
            &Point2::from(b_end),
            0.01,
        ) {
            Some(Point { loc1: SegmentPointLocation::OnEdge([_, from_start]), .. }) =>
                Some(a_start + from_start * (a_end - a_start)),
            _ => None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;
    use std::io;
    use bevy::math::{Vec2, Vec3};
    use bevy::prelude::Transform;
    use svg::Document;
    use crate::polygon::{Polygon, PolygonTransformBundle};

    fn save_svg(document: Document, stable_name: &str) -> io::Result<String> {
        let comparison_image = &format!("target/{stable_name}.svg");
        svg::save(comparison_image, &document).unwrap();

        let parent = current_dir()?;
        return Ok(format!("file:///{}", parent.join(comparison_image).display()));
    }

    #[test]
    fn test_sink_simple_subtract() {
        let left_operand = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(-4., -12.),
                Vec2::new(-4., -20.),
                Vec2::new(-12., -20.),
                Vec2::new(-12., -12.),
            ]),
            transform: Transform::from_xyz(4., 8., 0.)
                .with_scale(Vec3::splat(0.5)),
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

        let actual = left_operand.clone().sink(2., right_operand.clone());
        let expected = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(-4., -12.),
                Vec2::new(-4., -20.),
                Vec2::new(-12., -20.),
                Vec2::new(-12., -12.),
                Vec2::new(-10., -12.),
                Vec2::new(-10., -14.),
                Vec2::new(-6., -14.),
                Vec2::new(-6., -12.),
            ]),
            transform: Transform::from_xyz(4., 8., 0.)
                .with_scale(Vec3::splat(0.5)),
        };

        let scene = Document::new()
            .set("viewBox", (-3, -3, 6, 6))
            .add(actual.svg_path("red", 0.25))
            .add(expected.svg_path("green", 0.125))
            .add(left_operand.svg_path("black", 0.125 / 4.))
            .add(right_operand.svg_path("white", 0.125 / 4.))
            ;

        assert_eq!(actual, expected, "Visual: {:?}", save_svg(scene, "test_sink_simple_subtract"))
    }

    #[test]
    fn test_sink_double_subtract() {
        let left_operand = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(3., 2.),
                Vec2::new(3., 0.),
                Vec2::new(-2., 0.),
                Vec2::new(-2., 2.),
            ]),
            transform: Transform::from_xyz(0., 0., 0.),
        };

        let right_operand = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(2., 4.),
                Vec2::new(2., 1.),
                Vec2::new(1., 1.),
                Vec2::new(1., 3.),
                Vec2::new(0., 3.),
                Vec2::new(0., 1.),
                Vec2::new(-1., 1.),
                Vec2::new(-1., 4.),
            ]),
            transform: Transform::from_xyz(0., 0., 0.),
        };

        let actual = left_operand.clone().sink(2., right_operand.clone());
        let expected = PolygonTransformBundle {
            polygon: Polygon::from(vec![
                Vec2::new(3., 2.),
                Vec2::new(3., 0.),
                Vec2::new(-2., 0.),
                Vec2::new(-2., 2.),
                Vec2::new(-1., 2.),
                Vec2::new(-1., 1.),
                Vec2::new(0., 1.),
                Vec2::new(0., 2.),
                Vec2::new(1., 2.),
                Vec2::new(1., 1.),
                Vec2::new(2., 1.),
                Vec2::new(2., 2.),
            ]),
            transform: Transform::from_xyz(0., 0., 0.),
        };


        let scene = Document::new()
            .set("viewBox", (-3, -4, 7, 6))
            .add(actual.svg_path("red", 0.25))
            .add(expected.svg_path("green", 0.125))
            .add(left_operand.svg_path("black", 0.125 / 4.))
            .add(right_operand.svg_path("white", 0.125 / 4.))
            ;

        assert_eq!(actual, expected, "Visual: {:?}", save_svg(scene, "test_sink_double_subtract"))
    }
}