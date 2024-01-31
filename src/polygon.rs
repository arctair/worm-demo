use bevy::math::Vec2;
use bevy::prelude::Component;
use svg::Document;
use svg::node::element::Path;
use svg::node::element::path::Data;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Polygon {
    pub(crate) vertices: Vec<Vec2>,
}

impl Polygon {
    pub(crate) fn svg(&self) -> Path {
        let data = Data::new()
            .move_to((10, 10))
            .line_by((0, 50))
            .line_by((50, 0))
            .line_by((0, -50))
            .close();

        return Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 3)
            .set("d", data);
    }
}

impl Polygon {
    fn sink(self, area: f32, (s0, s1, s2, s3): (Vec2, Vec2, Vec2, Vec2)) -> Polygon { return self; }
}

impl From<Vec<Vec2>> for Polygon {
    fn from(vertices: Vec<Vec2>) -> Self {
        Polygon { vertices }
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;
    use svg::Document;
    use crate::polygon::Polygon;

    #[test]
    fn test_sink() {
        let left_operand = Polygon::from(vec![
            Vec2::new(2., 2.),
            Vec2::new(2., -2.),
            Vec2::new(-2., -2.),
            Vec2::new(-2., 2.),
        ]);

        let right_operand = (Vec2::new(1., 3.), Vec2::new(1., 1.), Vec2::new(-1., 1.), Vec2::new(-1., 3.));

        let actual = left_operand.sink(2., right_operand);
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

        let document = Document::new()
            .set("viewBox", (0, 0, 70, 70))
            .add(actual.svg())
            .add(expected.svg())
            ;

        svg::save("image.svg", &document).unwrap();

        assert_eq!(actual, expected)
    }
}