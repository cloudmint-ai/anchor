use super::{MustInto, Round, RoundDiv, Signed};
use crate::*;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

#[derive(Debug, Copy, Clone)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn into<U>(self) -> Point<U>
    where
        T: Into<U>,
    {
        Point {
            x: self.x.into(),
            y: self.y.into(),
        }
    }

    pub fn try_into<U>(self) -> Result<Point<U>>
    where
        T: TryInto<U, Error = Error>,
    {
        Ok(Point {
            x: self.x.try_into()?,
            y: self.y.try_into()?,
        })
    }

    pub fn must_into<U>(self) -> Point<U>
    where
        T: MustInto<U>,
    {
        Point {
            x: self.x.must_into(),
            y: self.y.must_into(),
        }
    }

    pub fn mid(point1: Point<T>, point2: Point<T>) -> Self
    where
        T: Add<T, Output = T> + Sub<T, Output = T> + Signed + RoundDiv<usize, Output = T> + Copy,
    {
        point1 + (point2 - point1) / 2
    }

    // 叉乘
    pub fn cross(self, other: Point<T>) -> T
    where
        T: Mul<T, Output = T> + Sub<T, Output = T> + Signed,
    {
        (self.x * other.y) - (self.y * other.x)
    }
}

impl<T> Point<T>
where
    T: Round + Copy,
{
    pub fn round(&self) -> Point<i64> {
        Point {
            x: self.x.round(),
            y: self.y.round(),
        }
    }
}

impl<T> PartialEq for Point<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl<T> Add<Point<T>> for Point<T>
where
    T: Add<T, Output = T>,
{
    type Output = Self;

    fn add(self, other: Point<T>) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T, U> AddAssign<Point<U>> for Point<T>
where
    T: AddAssign<U>,
{
    fn add_assign(&mut self, rhs: Point<U>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T> Sub<Point<T>> for Point<T>
where
    T: Sub<T, Output = T> + Signed,
{
    type Output = Self;

    fn sub(self, other: Point<T>) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T> SubAssign<Point<T>> for Point<T>
where
    T: SubAssign<T>,
{
    fn sub_assign(&mut self, rhs: Point<T>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

// 叉乘 用 cross 方法，Mul仅用于 向量 * 标量
impl<T, U> Mul<U> for Point<T>
where
    T: Mul<U, Output = T>,
    U: Copy,
{
    type Output = Point<T>;

    fn mul(self, rhs: U) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> Div<i64> for Point<T>
where
    T: RoundDiv<i64, Output = T>,
{
    type Output = Point<T>;

    fn div(self, other: i64) -> Self::Output {
        Self {
            x: self.x.round_div(other),
            y: self.y.round_div(other),
        }
    }
}

impl<T> Div<i32> for Point<T>
where
    T: RoundDiv<i32, Output = T>,
{
    type Output = Point<T>;

    fn div(self, other: i32) -> Self::Output {
        Self {
            x: self.x.round_div(other),
            y: self.y.round_div(other),
        }
    }
}

impl<T> Div<usize> for Point<T>
where
    T: RoundDiv<usize, Output = T>,
{
    type Output = Point<T>;

    fn div(self, other: usize) -> Self::Output {
        Self {
            x: self.x.round_div(other),
            y: self.y.round_div(other),
        }
    }
}

impl<T> Zeroable for Point<T>
where
    T: Zeroable,
{
    fn zero() -> Self {
        Self {
            x: T::zero(),
            y: T::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }
}
