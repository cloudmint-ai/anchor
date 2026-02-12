use super::{Point, Signed, Zeroable};
use crate::*;
use std::cmp::{max, min};
use std::fmt::Debug;
use std::ops::{Index, Mul, Range, Sub};

#[derive(Debug, Clone, PartialEq)]
pub struct Polygon<T> {
    pub min_x: T,
    pub min_x_point: Point<T>,
    pub max_x: T,
    pub max_x_point: Point<T>,
    pub min_y: T,
    pub min_y_point: Point<T>,
    pub max_y: T,
    pub max_y_point: Point<T>,
    vertices: Vec<Point<T>>, // len >= 3
}

impl<T> Index<usize> for Polygon<T> {
    type Output = Point<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vertices[index]
    }
}

impl<T> Index<Range<usize>> for Polygon<T> {
    type Output = [Point<T>];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.vertices[index]
    }
}

impl<T> Polygon<T> {
    pub fn new(vertices: Vec<Point<T>>) -> Result<Self>
    where
        T: Sub<Output = T> + Mul<Output = T> + Zeroable + Ord + Copy + Debug,
    {
        let n = vertices.len();
        if n < 3 {
            return Unexpected!("no enough points {}", n);
        }

        for i in 0..n {
            for j in i + 1..n {
                if j != (i + 1) % n && i != (j + 1) % n {
                    if is_lines_intersect(
                        vertices[i],
                        vertices[(i + 1) % n],
                        vertices[j],
                        vertices[(j + 1) % n],
                    ) {
                        return Unexpected!(
                            "lines [{:?}, {:?}] [{:?}, {:?}] intersect",
                            vertices[i],
                            vertices[(i + 1) % n],
                            vertices[j],
                            vertices[(j + 1) % n],
                        );
                    }
                }
            }
        }

        let mut min_x = vertices[0].x;
        let mut min_x_point = vertices[0];
        let mut max_x = vertices[0].x;
        let mut max_x_point = vertices[0];
        let mut min_y = vertices[0].y;
        let mut min_y_point = vertices[0];
        let mut max_y = vertices[0].y;
        let mut max_y_point = vertices[0];
        for i in 1..n {
            if vertices[i].x < min_x {
                min_x = vertices[i].x;
                min_x_point = vertices[i];
            }
            if vertices[i].x > max_x {
                max_x = vertices[i].x;
                max_x_point = vertices[i];
            }
            if vertices[i].y < min_y {
                min_y = vertices[i].y;
                min_y_point = vertices[i];
            }
            if vertices[i].y > max_y {
                max_y = vertices[i].y;
                max_y_point = vertices[i];
            }
        }
        Ok(Self {
            min_x,
            min_x_point,
            max_x,
            max_x_point,
            min_y,
            min_y_point,
            max_y,
            max_y_point,
            vertices,
        })
    }

    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    pub fn contains(&self, point: Point<T>) -> bool
    where
        T: Ord + Signed + Zeroable + Sub<Output = T> + Mul<Output = T> + Copy,
    {
        // 向右水平射线法，若射线和边的交点为奇数个，则点在多边形内
        let mut inside = false;
        let n = self.vertices.len();

        let mut same_y_upper_sides_count = 0;
        let mut same_y_lower_sides_count = 0;

        for i in 0..n {
            let j = if i == 0 { n - 1 } else { i - 1 };
            // 顶点属于多边形
            if self.vertices[i] == point || self.vertices[j] == point {
                return true;
            }
            // 叉积为零，则共线
            if (point - self.vertices[i]).cross(point - self.vertices[j]) == T::zero() {
                // 若 point 在 i, j 范围内，则点在线上，属于多边形
                if (min(self.vertices[i].x, self.vertices[j].x) <= point.x)
                    && (point.x <= max(self.vertices[i].x, self.vertices[j].x))
                {
                    return true;
                }
                // 否则，点在延长线上，不需要影响交叉计算
                continue;
            }

            if point.y == self.vertices[i].y {
                if point.x < self.vertices[i].x {
                    if point.y < self.vertices[j].y {
                        same_y_lower_sides_count += 1
                    }
                    if point.y > self.vertices[j].y {
                        same_y_upper_sides_count += 1
                    }
                }
                continue;
            }

            if point.y == self.vertices[j].y {
                if point.x < self.vertices[j].x {
                    if point.y < self.vertices[i].y {
                        same_y_lower_sides_count += 1
                    }
                    if point.y > self.vertices[i].y {
                        same_y_upper_sides_count += 1
                    }
                }
                continue;
            }

            // 确认了不会相等后，则进行具体的相交判断
            if (point.y > self.vertices[i].y) != (point.y > self.vertices[j].y) {
                // 避免 div 导致的精度问题，进行左右变换，并注意同乘后根据正负改变大于小于方向
                // let cross_point_x = self.vertices[i].x
                //     + (self.vertices[j].x - self.vertices[i].x) * (point.y - self.vertices[i].y)
                //         / (self.vertices[j].y - self.vertices[i].y);
                // if point.x < cross_point_x {
                //     inside = !inside;
                // }
                if self.vertices[j].y - self.vertices[i].y > T::zero() {
                    if (point.x - self.vertices[i].x) * (self.vertices[j].y - self.vertices[i].y)
                        < (self.vertices[j].x - self.vertices[i].x) * (point.y - self.vertices[i].y)
                    {
                        inside = !inside;
                    }
                }
                if self.vertices[j].y - self.vertices[i].y < T::zero() {
                    if (point.x - self.vertices[i].x) * (self.vertices[j].y - self.vertices[i].y)
                        > (self.vertices[j].x - self.vertices[i].x) * (point.y - self.vertices[i].y)
                    {
                        inside = !inside;
                    }
                }
            }
        }

        if same_y_upper_sides_count % 2 == 1 && same_y_lower_sides_count % 2 == 1 {
            inside = !inside;
        }
        inside
    }
}

#[derive(PartialEq)]
enum Orientation {
    Collinear,
    Clockwise,
    Counterclockwise,
}

fn is_lines_intersect<T>(p1: Point<T>, q1: Point<T>, p2: Point<T>, q2: Point<T>) -> bool
where
    T: Sub<Output = T> + Mul<Output = T> + Zeroable + Ord + Copy,
{
    let o1 = orientation(p1, q1, p2);
    let o2 = orientation(p1, q1, q2);
    let o3 = orientation(p2, q2, p1);
    let o4 = orientation(p2, q2, q1);

    if o1 != o2 && o3 != o4 {
        return true;
    }

    if o1 == Orientation::Collinear && on_segment(p1, p2, q1) {
        return true;
    }
    if o2 == Orientation::Collinear && on_segment(p1, q2, q1) {
        return true;
    }
    if o3 == Orientation::Collinear && on_segment(p2, p1, q2) {
        return true;
    }
    if o4 == Orientation::Collinear && on_segment(p2, q1, q2) {
        return true;
    }

    false
}

fn orientation<T>(p: Point<T>, q: Point<T>, r: Point<T>) -> Orientation
where
    T: Sub<Output = T> + Mul<Output = T> + Zeroable + Ord + Copy,
{
    let val = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);
    if val.is_zero() {
        return Orientation::Collinear;
    }
    if val > T::zero() {
        Orientation::Clockwise
    } else {
        Orientation::Counterclockwise
    }
}

fn on_segment<T>(p: Point<T>, q: Point<T>, r: Point<T>) -> bool
where
    T: Ord + Copy,
{
    q.x <= max(p.x, r.x) && q.x >= min(p.x, r.x) && q.y <= max(p.y, r.y) && q.y >= min(p.y, r.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test::case]
    fn test_contains() {
        let polygon = Polygon::new(vec![
            Point { x: 1, y: 1 },
            Point { x: 1, y: 3 },
            Point { x: 3, y: 3 },
            Point { x: 3, y: 5 },
            Point { x: 5, y: 5 },
            Point { x: 5, y: 1 },
        ])?;
        assert_eq!(polygon, polygon);
        assert!(polygon.contains(Point { x: 1, y: 1 }));
        assert!(polygon.contains(Point { x: 2, y: 2 }));
        assert!(!polygon.contains(Point { x: 0, y: 3 }));

        let polygon = Polygon::new(vec![
            Point { x: 1, y: 1 },
            Point { x: 3, y: 3 },
            Point { x: 5, y: 1 },
        ])?;
        assert_eq!(polygon, polygon);
        assert!(polygon.contains(Point { x: 1, y: 1 }));
        assert!(!polygon.contains(Point { x: 0, y: 1 }));
        assert!(polygon.contains(Point { x: 2, y: 2 }));
        assert!(polygon.contains(Point { x: 3, y: 3 }));
        assert!(!polygon.contains(Point { x: 2, y: 3 }));
        assert!(!polygon.contains(Point { x: 4, y: 3 }));

        let polygon = Polygon::new(vec![
            Point { x: 1, y: 1 },
            Point { x: 3, y: 3 },
            Point { x: 5, y: 5 },
            Point { x: 5, y: 1 },
        ])?;
        assert_eq!(polygon, polygon);
        assert!(polygon.contains(Point { x: 1, y: 1 }));
        assert!(!polygon.contains(Point { x: 0, y: 1 }));
        assert!(polygon.contains(Point { x: 2, y: 2 }));
        assert!(polygon.contains(Point { x: 3, y: 3 }));
        assert!(!polygon.contains(Point { x: 2, y: 3 }));
        assert!(polygon.contains(Point { x: 4, y: 3 }));
        assert!(!polygon.contains(Point { x: 4, y: 5 }));

        let polygon2 = Polygon::new(vec![
            Point { x: 1, y: 1 },
            Point { x: 3, y: 3 },
            Point { x: 5, y: 5 },
            Point { x: 5, y: 1 },
        ])?;
        assert_eq!(polygon, polygon2);

        let polygon3 = Polygon::new(vec![
            Point { x: 1, y: 1 },
            Point { x: 4, y: 4 },
            Point { x: 5, y: 5 },
            Point { x: 5, y: 1 },
        ])?;
        assert_ne!(polygon, polygon3);
    }
}
