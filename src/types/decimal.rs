use super::{MustInto, Round, RoundDiv, Signed, Zeroable};
use crate::*;
use rust_decimal::Decimal as RustDecimal;
use rust_decimal::prelude::*;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::ops::{Range, RangeInclusive};

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct Decimal(RustDecimal);

impl Decimal {
    pub const MAX: Self = Self(RustDecimal::MAX);
    pub const MIN: Self = Self(RustDecimal::MIN);
    pub const ZERO: Self = Self(RustDecimal::ZERO);
    pub const ONE: Self = Self(RustDecimal::ONE);
    pub const TWO: Self = Self(RustDecimal::TWO);
    pub const ONE_HUNDRED: Self = Self(RustDecimal::ONE_HUNDRED);
    pub const PI: Self = Self(RustDecimal::PI);

    pub fn from_x100(x100: i64) -> Self {
        Self(RustDecimal::new(x100, 2))
    }

    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    // 不应滥用，仅用于本 crate 内已经取整后的 Decimal 类型转换
    pub(crate) fn must_into(self) -> i64 {
        self.0.to_i64().unwrap()
    }

    pub fn floor(&self) -> i64 {
        Self(self.0.floor()).must_into()
    }

    pub fn ceil(&self) -> i64 {
        Self(self.0.ceil()).must_into()
    }

    pub fn round(self) -> i64 {
        self.round_decimal().must_into()
    }

    pub fn round_decimal(&self) -> Self {
        Self(self.0.round())
    }

    pub fn scale(&self) -> u32 {
        self.0.scale()
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn square(&self) -> Self {
        Self(self.0 * self.0)
    }

    pub fn sqrt(&self) -> Result<Self> {
        Ok(Self(self.0.sqrt().ok_or_else(none!())?))
    }

    pub fn sin(&self) -> Result<Self> {
        if let Some(result) = self.0.checked_sin() {
            Ok(Self(result))
        } else {
            Unexpected!("{:?} sin fail", self)
        }
    }

    pub fn cos(&self) -> Result<Decimal> {
        if let Some(result) = self.0.checked_cos() {
            Ok(Self(result))
        } else {
            Unexpected!("{:?} sin fail", self)
        }
    }

    // TODO refact with std::iter::Step
    pub fn range<R>(range: R) -> DecimalIterator
    where
        R: Into<DecimalIterator>,
    {
        range.into()
    }
}

impl std::fmt::Debug for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl From<RustDecimal> for Decimal {
    fn from(value: RustDecimal) -> Self {
        Self(value)
    }
}

impl From<i64> for Decimal {
    fn from(value: i64) -> Self {
        Self(RustDecimal::new(value, 0))
    }
}

impl From<i32> for Decimal {
    fn from(value: i32) -> Self {
        let value: i64 = value.into();
        Self::from(value)
    }
}

impl From<u8> for Decimal {
    fn from(value: u8) -> Self {
        let value: i64 = value.into();
        Self::from(value)
    }
}

impl From<usize> for Decimal {
    fn from(value: usize) -> Self {
        let value: i64 = value.must_into();
        Self::from(value)
    }
}

impl TryFrom<&str> for Decimal {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self> {
        Ok(Self(RustDecimal::from_str(value)?))
    }
}

impl TryFrom<String> for Decimal {
    type Error = Error;
    fn try_from(value: String) -> Result<Self> {
        Ok(Self(RustDecimal::from_str(&value)?))
    }
}

impl Into<String> for Decimal {
    fn into(self) -> String {
        self.0.to_string()
    }
}

impl Neg for Decimal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Add for Decimal {
    type Output = Self;

    fn add(self, other: Decimal) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl<T> AddAssign<T> for Decimal
where
    Decimal: Add<T, Output = Decimal>,
{
    fn add_assign(&mut self, other: T) {
        *self = *self + other
    }
}

impl Add<i64> for Decimal {
    type Output = Self;

    fn add(self, other: i64) -> Self::Output {
        Self(self.0 + RustDecimal::from(other))
    }
}

impl Add<i32> for Decimal {
    type Output = Self;

    fn add(self, other: i32) -> Self::Output {
        Self(self.0 + RustDecimal::from(other))
    }
}

impl Add<usize> for Decimal {
    type Output = Self;

    fn add(self, other: usize) -> Self::Output {
        Self(self.0 + RustDecimal::from(other))
    }
}

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, other: Decimal) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl<T> SubAssign<T> for Decimal
where
    Decimal: Sub<T, Output = Decimal>,
{
    fn sub_assign(&mut self, other: T) {
        *self = *self - other
    }
}

impl Sub<i64> for Decimal {
    type Output = Self;

    fn sub(self, other: i64) -> Self::Output {
        Self(self.0 - RustDecimal::from(other))
    }
}

impl Sub<i32> for Decimal {
    type Output = Self;

    fn sub(self, other: i32) -> Self::Output {
        Self(self.0 - RustDecimal::from(other))
    }
}

impl Sub<usize> for Decimal {
    type Output = Self;

    fn sub(self, other: usize) -> Self::Output {
        Self(self.0 - RustDecimal::from(other))
    }
}

impl Mul for Decimal {
    type Output = Self;

    fn mul(self, other: Decimal) -> Self::Output {
        Self(self.0 * other.0)
    }
}

impl<T> MulAssign<T> for Decimal
where
    Decimal: Mul<T, Output = Decimal>,
{
    fn mul_assign(&mut self, other: T) {
        *self = *self * other
    }
}

impl Mul<i64> for Decimal {
    type Output = Self;

    fn mul(self, other: i64) -> Self::Output {
        let other: Self = other.into();
        self * other
    }
}

impl Mul<Decimal> for i64 {
    type Output = Decimal;

    fn mul(self, other: Decimal) -> Self::Output {
        let s: Decimal = self.into();
        s * other
    }
}

impl Mul<i32> for Decimal {
    type Output = Self;

    fn mul(self, other: i32) -> Self::Output {
        let other: Self = other.into();
        self * other
    }
}

impl Mul<Decimal> for i32 {
    type Output = Decimal;

    fn mul(self, other: Decimal) -> Self::Output {
        let s: Decimal = self.into();
        s * other
    }
}

impl Mul<usize> for Decimal {
    type Output = Self;

    fn mul(self, other: usize) -> Self::Output {
        let other: Self = other.into();
        self * other
    }
}

impl Mul<Decimal> for usize {
    type Output = Decimal;

    fn mul(self, other: Decimal) -> Self::Output {
        let s: Decimal = self.into();
        s * other
    }
}

impl Div for Decimal {
    type Output = Self;

    fn div(self, other: Decimal) -> Self::Output {
        Self(self.0 / other.0)
    }
}

impl<T> DivAssign<T> for Decimal
where
    Decimal: Div<T, Output = Decimal>,
{
    fn div_assign(&mut self, other: T) {
        *self = *self / other
    }
}

impl Div<i64> for Decimal {
    type Output = Self;

    fn div(self, other: i64) -> Self::Output {
        let other: Decimal = other.into();
        self / other
    }
}

impl Div<i32> for Decimal {
    type Output = Self;

    fn div(self, other: i32) -> Self::Output {
        let other: Decimal = other.into();
        self / other
    }
}

impl Div<usize> for Decimal {
    type Output = Self;

    fn div(self, other: usize) -> Self::Output {
        let other: Decimal = other.into();
        self / other
    }
}

impl Round for Decimal {
    fn round(self) -> i64 {
        self.round()
    }
}

impl RoundDiv for Decimal {
    type Output = Self;
    fn round_div(self, other: Decimal) -> Self::Output {
        self / other
    }
}

impl RoundDiv<i64> for Decimal {
    type Output = Self;
    fn round_div(self, other: i64) -> Self::Output {
        self / other
    }
}

impl RoundDiv<i32> for Decimal {
    type Output = Self;
    fn round_div(self, other: i32) -> Self::Output {
        self / other
    }
}

impl RoundDiv<usize> for Decimal {
    type Output = Self;
    fn round_div(self, other: usize) -> Self::Output {
        self / other
    }
}

impl PartialEq<i64> for Decimal {
    fn eq(&self, other: &i64) -> bool {
        self.eq(&Decimal::from(*other))
    }
}

impl PartialOrd<i64> for Decimal {
    fn partial_cmp(&self, other: &i64) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Decimal::from(*other))
    }
}

impl PartialEq<i32> for Decimal {
    fn eq(&self, other: &i32) -> bool {
        self.eq(&Decimal::from(*other))
    }
}

impl PartialOrd<i32> for Decimal {
    fn partial_cmp(&self, other: &i32) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Decimal::from(*other))
    }
}

impl PartialEq<usize> for Decimal {
    fn eq(&self, other: &usize) -> bool {
        self.eq(&Decimal::from(*other))
    }
}

impl PartialOrd<usize> for Decimal {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Decimal::from(*other))
    }
}

impl Signed for Decimal {}

impl Zeroable for Decimal {
    fn zero() -> Self {
        Decimal::ZERO
    }

    fn is_zero(&self) -> bool {
        self.is_zero()
    }
}

impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let json_number =
            json::Number::from_str(&self.0.to_string()).map_err(serde::ser::Error::custom)?;
        serializer.serialize_newtype_struct("Decimal", &json_number)
    }
}

impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = json::Value::deserialize(deserializer)?;
        match value {
            json::Value::Number(number) => {
                // 通过 `to_string` 获取数字的字符串表示，转换为 Decimal
                let decimal_value =
                    Decimal::try_from(number.to_string()).map_err(serde::de::Error::custom)?;
                Ok(decimal_value)
            }
            _ => Err(serde::de::Error::custom("Expected a number")),
        }
    }
}

pub struct DecimalIterator {
    start: Decimal,
    current: Decimal,
    end: Option<Decimal>,
    inclusive: bool,
    step: Decimal,
}

impl DecimalIterator {
    pub fn rev(self) -> Result<Self> {
        if !self.inclusive {
            return Unexpected!("revert iterator without inclusive");
        }
        if let Some(end) = self.end {
            Ok(Self {
                start: end,
                current: end,
                end: Some(self.start),
                inclusive: true,
                step: -Decimal::ONE,
            })
        } else {
            Unexpected!("revert iterator without end")
        }
    }
}

impl Iterator for DecimalIterator {
    type Item = Decimal;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(end) = self.end {
            if self.current > end || (!self.inclusive && self.current == end) {
                return None;
            }
        }
        let result = self.current;
        self.current += self.step;
        Some(result)
    }
}

impl From<RangeInclusive<Decimal>> for DecimalIterator {
    fn from(value: RangeInclusive<Decimal>) -> Self {
        Self {
            start: *value.start(),
            current: *value.start(),
            end: Some(*value.end()),
            inclusive: true,
            step: Decimal::ONE,
        }
    }
}

impl From<Range<Decimal>> for DecimalIterator {
    fn from(value: Range<Decimal>) -> Self {
        Self {
            start: value.start,
            current: value.start,
            end: Some(value.end),
            inclusive: false,
            step: Decimal::ONE,
        }
    }
}

#[macro_export]
macro_rules! dec {
    ($val:expr) => {
        Decimal::try_from(stringify!($val)).unwrap()
    };
}

tests! {
    fn test_decimal() {
        let value = Decimal::try_from("11.22")?;
        let value_string: String = value.into();
        assert_eq!(value_string, "11.22");
        assert_eq!(format!("{:?}", value), "11.22");

        let mut value = Decimal::from(1122);
        value = value / 100;
        let value_string: String = value.into();
        assert_eq!(value_string, "11.22");
        assert_eq!(format!("{:?}", value), "11.22");

        let mut value = Decimal::from(1120);
        value = value / 100;
        let value_string: String = value.into();
        assert_eq!(value_string, "11.20");
        assert_eq!(format!("{:?}", value), "11.20");

        let value = Decimal::from(4);
        assert_eq!(value.sqrt()?, Decimal::from(2));

        let value = Decimal::try_from("0.04")?;
        assert_eq!(value.sqrt()? * 10, Decimal::from(2));

        let value = Decimal::try_from("0.0")?;
        assert_eq!(value.sqrt()?, Decimal::from(0));

        let value = Decimal::try_from("33.55")?;
        let serialized = json::to_string(&value)?;
        assert_eq!(serialized, "33.55");

        let deserialized: Decimal = json::from_str(&serialized)?;
        assert_eq!(deserialized, value);

        let value = Decimal::from(55);
        let serialized = json::to_string(&value)?;
        assert_eq!(serialized, "55");

        let deserialized: Decimal = json::from_str(&serialized)?;
        assert_eq!(deserialized, value);

        let value = Decimal::from(0);
        assert_eq!(value, value.sin()?);

        let value = Decimal::from(0);
        assert_eq!(value.cos()?, Decimal::ONE);
    }
}
