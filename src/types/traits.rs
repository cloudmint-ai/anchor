use std::fmt::Debug;

pub trait Signed {}
impl Signed for i64 {}

pub trait Zeroable {
    fn zero() -> Self;
    fn is_zero(&self) -> bool;
}

impl Zeroable for i32 {
    fn zero() -> Self {
        0
    }

    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl Zeroable for i64 {
    fn zero() -> Self {
        0
    }

    fn is_zero(&self) -> bool {
        *self == 0
    }
}

// 主要用于替换范围更窄风险更高的 as 操作
// 在可以用 ? 的场景下应该使用 TryInto
pub trait MustInto<T> {
    fn must_into(self) -> T;
}

impl<T, U, E> MustInto<T> for U
where
    U: TryInto<T, Error = E> + Debug,
    E: Debug,
{
    fn must_into(self) -> T {
        self.try_into().unwrap()
    }
}

pub trait Round {
    fn round(self) -> i64;
}

impl Round for i64 {
    fn round(self) -> i64 {
        self
    }
}

impl Round for i32 {
    fn round(self) -> i64 {
        self.into()
    }
}

impl Round for usize {
    fn round(self) -> i64 {
        self.must_into()
    }
}

// 对于需要截断小数部分的类型，进行四舍五入
// 对于无需截断的类型，直接进行除法，《并非》进行取整
pub trait RoundDiv<T = Self> {
    type Output;
    fn round_div(self, other: T) -> Self::Output;
}

impl RoundDiv for i64 {
    type Output = Self;

    fn round_div(self, other: i64) -> Self::Output {
        if other == 0 {
            panic!("cannot div zero");
        }
        let quotient = self / other;
        let remainder = self % other;
        if (remainder.abs() * 2) >= other.abs() {
            if self.is_positive() == other.is_positive() {
                quotient + 1
            } else {
                quotient - 1
            }
        } else {
            quotient
        }
    }
}

impl RoundDiv<i32> for i64 {
    type Output = Self;

    fn round_div(self, other: i32) -> Self::Output {
        let other: i64 = other.into();
        self.round_div(other)
    }
}

impl RoundDiv<usize> for i64 {
    type Output = Self;

    fn round_div(self, other: usize) -> Self::Output {
        let other: i64 = other.try_into().unwrap();
        self.round_div(other)
    }
}

impl RoundDiv for i32 {
    type Output = Self;

    fn round_div(self, other: i32) -> Self::Output {
        if other == 0 {
            panic!("cannot div zero");
        }
        let quotient = self / other;
        let remainder = self % other;
        if (remainder.abs() * 2) >= other.abs() {
            if self.is_positive() == other.is_positive() {
                quotient + 1
            } else {
                quotient - 1
            }
        } else {
            quotient
        }
    }
}

impl RoundDiv<usize> for i32 {
    type Output = Self;

    fn round_div(self, other: usize) -> Self::Output {
        let other: i32 = other.try_into().unwrap();
        self.round_div(other)
    }
}

impl RoundDiv for usize {
    type Output = Self;
    fn round_div(self, other: usize) -> Self::Output {
        if other == 0 {
            panic!("cannot div zero");
        }
        let quotient = self / other;
        let remainder = self % other;
        if remainder * 2 >= other {
            quotient + 1
        } else {
            quotient
        }
    }
}

pub trait Mean<T> {
    fn mean(&self) -> T;
    fn round_mean(&self) -> T;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test::case]
    fn test_round_div() {
        let mut i: i64;
        i = 6;
        assert_eq!(i.round_div(2), 3i64);
        i = 10;
        assert_eq!(i.round_div(3), 3i64);
        i = 7;
        assert_eq!(i.round_div(2), 4i64);
        i = -7;
        assert_eq!(i.round_div(2), -4i64);
        i = 8;
        assert_eq!(i.round_div(2), 4i64);

        let mut i: usize;
        i = 6;
        assert_eq!(i.round_div(2), 3);
        i = 10;
        assert_eq!(i.round_div(3), 3);
        i = 7;
        assert_eq!(i.round_div(2), 4);
        i = 8;
        assert_eq!(i.round_div(2), 4);

        assert_eq!(6.round_div(2), 3);
        assert_eq!(6.round_div(2i64), 3);
        assert_eq!(10.round_div(3), 3);
        assert_eq!(10.round_div(3i64), 3);
        assert_eq!(7.round_div(2), 4);
        assert_eq!(7.round_div(2i64), 4);
        assert_eq!(-7.round_div(2i64), -4);
        assert_eq!(8.round_div(2), 4);
        assert_eq!(8.round_div(2i64), 4);
    }
}
