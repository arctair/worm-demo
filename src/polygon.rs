use bevy::math::Vec2;
use bevy::prelude::Component;
use bevy_rapier2d::na::Point2;
use bevy_rapier2d::parry::shape::SegmentPointLocation;
use bevy_rapier2d::parry::utils::segments_intersection2d;
use bevy_rapier2d::parry::utils::SegmentsIntersection::{Point, Segment};
use svg::node::element::Path;
use svg::node::element::path::Data;

#[derive(Clone, Component, Debug, PartialEq)]
pub(crate) struct Polygon {
    pub(crate) vertices: Vec<Vec2>,
}

impl Polygon {
    pub(crate) fn svg_path(&self, stroke: &str, stroke_width: f64) -> Path {
        return Path::new()
            .set("fill", "none")
            .set("stroke", stroke)
            .set("stroke-width", stroke_width)
            .set("d", self.svg_path_data());
    }

    pub(crate) fn svg_path_data(&self) -> Data {
        let mut data = Data::new()
            .move_to((self.vertices[0].x, -self.vertices[0].y));

        for vertex in self.vertices.iter().skip(1) {
            data = data.line_to((vertex.x, -vertex.y));
        }

        return data.close();
    }
}

#[derive(Clone, Copy, Debug)]
enum TraceMode {
    TracingSelf,
    TracingBounds,
}

impl Polygon {
    fn sink(self, area: f32, bounds: Self) -> Self {
        let mut new_vertices = vec![];
        let mut trace_mode = TraceMode::TracingSelf;
        let mut start_index = 0;
        let mut end_index = 1;
        let mut start_bounds_index = 1;
        let mut end_bounds_index = 0;

        while new_vertices.is_empty() || start_index != 0 || match trace_mode {
            TraceMode::TracingBounds => true,
            _ => false
        } {
            let mut intersection = None;
            match trace_mode {
                TraceMode::TracingSelf => {
                    new_vertices.push(self.vertices[start_index]);
                    for _ in 0..bounds.vertices.len() {
                        let start = self.vertices[start_index];
                        let end = self.vertices[end_index];
                        let start_bounds = bounds.vertices[start_bounds_index];
                        intersection = segments_intersection2d(
                            &Point2::from(start),
                            &Point2::from(end),
                            &Point2::from(start_bounds),
                            &Point2::from(bounds.vertices[end_bounds_index]),
                            0.01,
                        ).and_then(|intersection| {
                            if (end.x - start.x) * (start_bounds.y - start.y) - (end.y - start.y) * (start_bounds.x - start.x) > 0. {
                                Some(intersection)
                            } else {
                                None
                            }
                        });
                        if intersection.is_some() { break; }

                        start_bounds_index = end_bounds_index;
                        end_bounds_index = (end_bounds_index + bounds.vertices.len() - 1) % bounds.vertices.len();
                    }
                }
                TraceMode::TracingBounds => {
                    new_vertices.push(bounds.vertices[start_bounds_index]);
                    for _ in 0..self.vertices.len() {
                        intersection = segments_intersection2d(
                            &Point2::from(self.vertices[start_index]),
                            &Point2::from(self.vertices[end_index]),
                            &Point2::from(bounds.vertices[start_bounds_index]),
                            &Point2::from(bounds.vertices[end_bounds_index]),
                            0.01,
                        );
                        if intersection.is_some() { break; }

                        start_index = end_index;
                        end_index = (end_index + 1) % self.vertices.len();
                    }
                }
            }


            match intersection {
                Some(Point { loc1: location, .. }) => {
                    match location {
                        SegmentPointLocation::OnVertex(_) => {}
                        SegmentPointLocation::OnEdge([_, from_start]) => {
                            println!("pushing intersection {start_index}-{end_index} {start_bounds_index}-{end_bounds_index} {from_start}");
                            let start = self.vertices[start_index];
                            let end = self.vertices[end_index];
                            new_vertices.push(start + from_start * (end - start));

                            match trace_mode {
                                TraceMode::TracingSelf => trace_mode = TraceMode::TracingBounds,
                                TraceMode::TracingBounds => trace_mode = TraceMode::TracingSelf,
                            }
                        }
                    }
                }
                Some(Segment { .. }) => {}
                None => {}
            }

            match trace_mode {
                TraceMode::TracingSelf => {
                    start_index = end_index;
                    end_index = (end_index + 1) % self.vertices.len();
                }
                TraceMode::TracingBounds => {
                    start_bounds_index = end_bounds_index;
                    end_bounds_index = (end_bounds_index + bounds.vertices.len() - 1) % bounds.vertices.len();
                }
            }
        }

        return Self::from(new_vertices);
    }
}

impl From<Vec<Vec2>> for Polygon {
    fn from(vertices: Vec<Vec2>) -> Self {
        Polygon { vertices }
    }
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;
    use std::io;
    use bevy::math::Vec2;
    use svg::Document;
    use crate::polygon::Polygon;

    fn save_svg(document: Document, stable_name: &str) -> io::Result<String> {
        let comparison_image = &format!("target/{stable_name}.svg");
        svg::save(comparison_image, &document).unwrap();

        let parent = current_dir()?;
        return Ok(format!("file:///{}", parent.join(comparison_image).display()));
    }

    #[test]
    fn test_sink_simple_subtract() {
        let left_operand = Polygon::from(vec![
            Vec2::new(2., 2.),
            Vec2::new(2., -2.),
            Vec2::new(-2., -2.),
            Vec2::new(-2., 2.),
        ]);

        let right_operand = Polygon::from(vec![
            Vec2::new(1., 3.),
            Vec2::new(1., 1.),
            Vec2::new(-1., 1.),
            Vec2::new(-1., 3.),
        ]);

        let actual = left_operand.clone().sink(2., right_operand.clone());
        let expected = Polygon::from(vec![
            Vec2::new(2., 2.),
            Vec2::new(2., -2.),
            Vec2::new(-2., -2.),
            Vec2::new(-2., 2.),
            Vec2::new(-1., 2.),
            Vec2::new(-1., 1.),
            Vec2::new(1., 1.),
            Vec2::new(1., 2.),
        ]);

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
        let left_operand = Polygon::from(vec![
            Vec2::new(3., 2.),
            Vec2::new(3., 0.),
            Vec2::new(-2., 0.),
            Vec2::new(-2., 2.),
        ]);

        let right_operand = Polygon::from(vec![
            Vec2::new(2., 4.),
            Vec2::new(2., 1.),
            Vec2::new(1., 1.),
            Vec2::new(1., 3.),
            Vec2::new(0., 3.),
            Vec2::new(0., 1.),
            Vec2::new(-1., 1.),
            Vec2::new(-1., 4.),
        ]);

        let actual = left_operand.clone().sink(2., right_operand.clone());
        let expected = Polygon::from(vec![
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
        ]);

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