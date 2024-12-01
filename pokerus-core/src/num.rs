use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
};
#[repr(transparent)]
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct Percentage<T>(T);
impl<T> Percentage<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
    pub fn copy_inner(&self) -> T
    where
        T: Copy,
    {
        self.0
    }
}
impl<T> PartialEq<T> for Percentage<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
    }
}
impl<T> PartialOrd<T> for Percentage<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}
impl<T> Percentage<T> {
    pub fn bound(self) -> Option<BoundedPercentage<T>>
    where
        T: FullScale + PartialOrd,
    {
        BoundedPercentage::from_full_scale(self.0)
    }
}
#[repr(transparent)]
pub struct BoundedPercentage<T>(T);
impl<T> BoundedPercentage<T> {
    pub fn from_full_scale(value: T) -> Option<BoundedPercentage<T>>
    where
        T: FullScale + PartialOrd,
    {
        value.is_bounded().then_some(BoundedPercentage(value))
    }
}
impl<T> PartialEq<T> for BoundedPercentage<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
    }
}
impl<T> PartialOrd<T> for BoundedPercentage<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}
pub trait FullScale {
    fn origin(&self) -> &Self;
    fn origin_cmp(&self) -> Option<Ordering>
    where
        Self: PartialOrd,
    {
        self.partial_cmp(self.origin())
    }
    fn unity(&self) -> &Self;
    fn unity_cmp(&self) -> Option<Ordering>
    where
        Self: PartialOrd,
    {
        self.partial_cmp(self.unity())
    }
    fn is_bounded(&self) -> bool
    where
        Self: PartialOrd,
    {
        let Some(lower) = self.origin_cmp() else {
            return false;
        };
        if matches!(lower, Ordering::Less) {
            return false;
        };
        let Some(upper) = self.unity_cmp() else {
            return false;
        };
        if matches!(upper, Ordering::Greater) {
            return false;
        };
        true
    }
    fn negative_unity<'a>(&'a self) -> <&'a Self as Neg>::Output
    where
        &'a Self: Neg,
    {
        self.unity().neg()
    }
}
impl FullScale for f32 {
    fn origin(&self) -> &Self {
        &0.
    }
    fn unity(&self) -> &Self {
        &1.
    }
}
pub trait IntoPercentage<T>: Sized {
    fn into_bounded_percentage(self) -> Option<BoundedPercentage<T>>
    where
        T: FullScale + PartialOrd,
    {
        self.into_percentage().bound()
    }
    fn into_percentage(self) -> Percentage<T>;
}
impl IntoPercentage<f32> for f32 {
    fn into_percentage(self) -> Percentage<Self> {
        Percentage(self)
    }
}
impl Add<Percentage<f32>> for Percentage<f32> {
    type Output = Percentage<f32>;
    fn add(self, rhs: Percentage<f32>) -> Self::Output {
        Percentage(self.0.add(rhs.0))
    }
}
impl Add<&Percentage<f32>> for Percentage<f32> {
    type Output = Percentage<f32>;
    fn add(self, rhs: &Percentage<f32>) -> Self::Output {
        Percentage(self.0.add(rhs.0))
    }
}
impl Mul<Percentage<f32>> for Percentage<f32> {
    type Output = Percentage<f32>;
    fn mul(self, rhs: Percentage<f32>) -> Self::Output {
        Percentage(self.0.mul(rhs.0))
    }
}
impl Mul<&Percentage<f32>> for Percentage<f32> {
    type Output = Percentage<f32>;
    fn mul(self, rhs: &Percentage<f32>) -> Self::Output {
        Percentage(self.0.mul(rhs.0))
    }
}
impl IntoPercentage<f32> for &f32 {
    fn into_percentage(self) -> Percentage<f32> {
        Percentage(*self)
    }
}
impl IntoPercentage<f64> for f64 {
    fn into_percentage(self) -> Percentage<Self> {
        Percentage(self)
    }
}
impl IntoPercentage<f64> for &f64 {
    fn into_percentage(self) -> Percentage<f64> {
        Percentage(*self)
    }
}
impl FullScale for u8 {
    fn origin(&self) -> &Self {
        &0
    }
    fn unity(&self) -> &Self {
        &u8::MAX
    }
}
impl IntoPercentage<u8> for u8 {
    fn into_percentage(self) -> Percentage<u8> {
        Percentage(self)
    }
}
pub struct Constu8InclusiveRange<const A: u8, const B: u8>(u8);
impl<const A: u8, const B: u8> Constu8InclusiveRange<A, B> {
    pub const fn new(value: u8) -> Self {
        if A >= B {
            panic!("A cannot be lower than B");
        };
        if value < A {
            panic!("value cannot be lower than A");
        };
        if value > B {
            panic!("value cannot be greater than B")
        };

        Constu8InclusiveRange(value)
    }
    pub const fn is_bounded_below_exclusive(&self) -> bool {
        self.0 > A
    }
    pub const fn is_bounded_below_inclusive(&self) -> bool {
        self.0 >= A
    }
    pub const fn is_bounded_above_exclusive(&self) -> bool {
        self.0 < B
    }
    pub const fn is_bounded_above_inclusive(&self) -> bool {
        self.0 <= B
    }
}
impl<const A: u8, const B: u8> Add for Constu8InclusiveRange<A, B> {
    type Output = Option<Self>;
    fn add(self, rhs: Self) -> Self::Output {
        let value = self.0.add(rhs.0);
        value.le(&B).then_some(Constu8InclusiveRange(value))
    }
}
impl<const A: u8, const B: u8> Sub for Constu8InclusiveRange<A, B> {
    type Output = Option<Self>;
    fn sub(self, rhs: Self) -> Self::Output {
        let value = self.0.sub(rhs.0);
        value.ge(&A).then_some(Constu8InclusiveRange(value))
    }
}
