use crate::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PointResource {
    pub x: usize,
    pub y: usize,
}

impl Into<Point<usize>> for PointResource {
    fn into(self) -> Point<usize> {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<Point<usize>> for PointResource {
    fn from(value: Point<usize>) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<Vec<PointResource>> for Polygon<i64> {
    fn from(value: Vec<PointResource>) -> Self {
        let vertices: Vec<Point<i64>> = value
            .into_iter()
            .map(|point_resource| Point {
                x: point_resource.x.must_into(),
                y: point_resource.y.must_into(),
            })
            .collect();

        Polygon::new(vertices).expect("Failed to create Polygon")
    }
}
