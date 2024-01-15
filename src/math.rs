use nalgebra::{Matrix2, Matrix3, Matrix4, Vector2, Vector3, Vector4};

#[cfg(not(feature = "f64"))]
pub type Fl = f32;
#[cfg(not(features = "f64"))]
pub use std::f32 as std_fl;
#[cfg(feature = "f64")]
pub type Fl = f64;
#[cfg(features = "f64")]
pub use std::f64 as std_fl;

pub trait IntoFl {
    fn into_fl(self) -> Fl;
}

macro_rules! impl_into_fl {
    ($ty:ident) => {
        impl IntoFl for $ty {
            fn into_fl(self) -> Fl {
                self as Fl
            }
        }
    };
}

impl_into_fl!(f32);
impl_into_fl!(f64);
impl_into_fl!(u8);
impl_into_fl!(i8);
impl_into_fl!(u16);
impl_into_fl!(i16);
impl_into_fl!(u32);
impl_into_fl!(i32);
impl_into_fl!(u64);
impl_into_fl!(i64);
impl_into_fl!(u128);
impl_into_fl!(i128);
impl_into_fl!(usize);
impl_into_fl!(isize);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vec2(pub Vector2<Fl>);
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vec3(pub Vector3<Fl>);
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vec4(pub Vector4<Fl>);
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Mat2(pub Matrix2<Fl>);
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Mat3(pub Matrix3<Fl>);
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Mat4(pub Matrix4<Fl>);

macro_rules! impl_vec_n {
    ( $vec:ident, $inner:ident; $( $name:ident: $ty_name:ident ),* ) => {
        impl $vec {
            #[inline(always)]
            pub fn new( $($name: impl IntoFl,)* ) -> Self {
                Self($inner::new( $($name.into_fl(),)* ))
            }
            $(
                #[inline(always)]
                pub fn $name(&self) -> Fl {
                    self.0.$name
                }
            )*

        }
        impl<$($ty_name: IntoFl,)*> From<($($ty_name,)*)> for $vec {
            #[inline(always)]
            fn from(($($name,)*): ($($ty_name,)*)) -> Self {
                Self::new($($name,)*)
            }
        }

        impl ::std::ops::Mul<$vec> for $vec {
            type Output = $vec;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self::new($(self.$name() * rhs.$name(),)*)
            }
        }

        impl ::std::ops::Mul<Fl> for $vec {
            type Output = $vec;

            #[inline(always)]
            fn mul(self, rhs: Fl) -> Self::Output {
                Self(self.0 * rhs)
            }
        }

        impl ::std::ops::Div<$vec> for $vec {
            type Output = $vec;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self::new($(self.$name() / rhs.$name(),)*)
            }
        }

        impl ::std::ops::Div<Fl> for $vec {
            type Output = $vec;

            #[inline(always)]
            fn div(self, rhs: Fl) -> Self::Output {
                Self(self.0 / rhs)
            }
        }
    };
}

impl_vec_n!(Vec2, Vector2; x: T, y: U);
impl_vec_n!(Vec3, Vector3; x: T, y: U, z: V);
impl_vec_n!(Vec4, Vector4; x: T, y: U, z: V, w: W);

impl Mat4 {
    pub fn ident() -> Self {
        Mat4(Matrix4::identity())
    }
}

impl Mat3 {
    pub fn ident() -> Self {
        Mat3(Matrix3::identity())
    }
}

impl Mat2 {
    pub fn ident() -> Self {
        Mat2(Matrix2::identity())
    }
}
