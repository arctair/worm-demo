use bevy::prelude::{Component, Transform};
use bevy::math::{Vec2, Vec3};
use bevy::render::render_resource::encase::private::RuntimeSizedArray;
use bevy_rapier2d::na::Point2;
use bevy_rapier2d::parry::math::Real;
use bevy_rapier2d::parry::transformation::convex_polygons_intersection_points;
use bevy_rapier2d::parry::utils::{segments_intersection2d, SegmentsIntersection};
use itertools::Itertools;

#[derive(Component)]
pub struct Geometry {
    local_vertices: Vec<Vec2>,
}

impl Geometry {
    pub fn new(local_vertices: Vec<Vec2>) -> Geometry {
        Geometry { local_vertices }
    }

    pub fn subtract((t0, g0): (&Transform, &Geometry), (t1, g1): (&Transform, &Geometry)) -> Vec<Vec2> {
        let mut left = g0.vertices(&t0);
        let right = g1.vertices(&t1);

        let mut result = vec![];
        let intersection: &mut Vec<Point2<Real>> = &mut vec![];
        convex_polygons_intersection_points(&left, &right, intersection);

        if intersection.is_empty() {
            return Geometry::perry2bevy(left);
        }

        let starting_index: usize = 0;
        left.push(left[0]);
        intersection.push(intersection[0]);

        let mut subtracting = false;
        let mut index = starting_index;
        loop {
            if subtracting {
                result.push(intersection[index]);

                let next_intersection_edge_index = (index + 1) % (intersection.len() - 1);
                for (edge_index, edge) in left.iter().tuple_windows::<(&Point2<Real>, &Point2<Real>)>().enumerate() {
                    let next_edge_index = (edge_index + 1) % (left.len() - 1);
                    let check = segments_intersection2d(edge.0, edge.1, &intersection[index], &intersection[next_intersection_edge_index], 0.001);
                    match check {
                        Some(SegmentsIntersection::Segment { .. }) => {
                            subtracting = false;
                            index = next_edge_index;
                            break;
                        }
                        _ => {}
                    }
                }

                if subtracting {
                    index = next_intersection_edge_index
                }
            } else {
                result.push(left[index]);

                let next_left_index = (index + 1) % (left.len() - 1);
                for (intersection_edge_index, intersection_edge) in intersection.iter().tuple_windows::<(&Point2<Real>, &Point2<Real>)>().enumerate() {
                    let next_intersection_edge_index = (intersection_edge_index + 1) % (intersection.len() - 1);
                    let check = segments_intersection2d(&left[index], &left[next_left_index], intersection_edge.0, intersection_edge.1, 0.001);
                    match check {
                        Some(SegmentsIntersection::Segment { .. }) => {
                            subtracting = true;
                            index = next_intersection_edge_index;
                            break;
                        }
                        _ => {}
                    }
                }

                if !subtracting {
                    index = next_left_index
                }
            }

            if !subtracting && index == starting_index {
                break;
            }
        }

        return Geometry::perry2bevy(result);
    }

    fn perry2bevy(perry: Vec<Point2<Real>>) -> Vec<Vec2> {
        perry.iter().map(|p| Vec2::new(p.x, p.y)).collect()
    }

    fn vertices(&self, transform: &Transform) -> Vec<Point2<Real>> {
        self.local_vertices.iter().map(|local_point_vec2| {
            let local_point_vec3 = Vec3::new(local_point_vec2.x, local_point_vec2.y, 0.);
            let global_point_vec3 = transform.transform_point(local_point_vec3);
            Point2::new(global_point_vec3.x, global_point_vec3.y)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_subtract_empty() {
        let actual = Geometry::subtract(
            (&Transform::default(), &Geometry::new(vec![])),
            (&Transform::default(), &Geometry::new(vec![])),
        );
        let expected = vec![];

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_non_intersecting_triangles() {
        let left_vertices = vec![
            Vec2::new(0., 0.),
            Vec2::new(0., 1.),
            Vec2::new(1., 0.),
        ];
        let actual = Geometry::subtract(
            (&Transform::default(), &Geometry::new(left_vertices.clone())),
            (&Transform::default(), &Geometry::new(vec![
                Vec2::new(2., 0.),
                Vec2::new(2., 1.),
                Vec2::new(3., 0.),
            ])),
        );

        assert_eq!(actual, left_vertices);
    }

    #[test]
    fn test_triangle_clips_triangle() {
        let actual = Geometry::subtract(
            (&Transform::default(), &Geometry::new(vec![
                Vec2::new(0., 0.),
                Vec2::new(0., 2.),
                Vec2::new(2., 0.),
            ])),
            (&Transform::default(), &Geometry::new(vec![
                Vec2::new(1., 0.),
                Vec2::new(1., 2.),
                Vec2::new(3., 0.),
            ])),
        );
        let expected = vec![
            Vec2::new(0., 0.),
            Vec2::new(0., 2.),
            Vec2::new(1., 1.),
            Vec2::new(1., 0.),
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_intersect_middle_edge() {
        let actual = Geometry::subtract(
            (&Transform::default(), &Geometry::new(vec![
                Vec2::new(0., 0.),
                Vec2::new(0., 2.),
                Vec2::new(4., 2.),
                Vec2::new(4., 0.),
            ])),
            (&Transform::default(), &Geometry::new(vec![
                Vec2::new(0., 3.),
                Vec2::new(4., 3.),
                Vec2::new(2., 1.),
            ])),
        );
        let expected = vec![
            Vec2::new(0., 0.),
            Vec2::new(0., 2.),
            Vec2::new(1., 2.),
            Vec2::new(2., 1.),
            Vec2::new(3., 2.),
            Vec2::new(4., 2.),
            Vec2::new(4., 0.),
        ];

        assert_eq!(actual, expected);
    }
}