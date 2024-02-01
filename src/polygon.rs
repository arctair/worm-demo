use bevy::math::Vec2;
use bevy::prelude::Component;
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

impl Polygon {
    fn sink(self, area: f32, bounds: Polygon) -> Polygon { return self; }
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
    fn test_sink() {
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

        assert_eq!(actual, expected, "Visual: {:?}", save_svg(scene, "test_sink"))
    }
}