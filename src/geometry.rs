pub use glam::{vec2, vec3, UVec2, UVec3, Vec2 as Vector2, Vec3 as Vector3};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Triangle<T> {
    pub a: T,
    pub b: T,
    pub c: T,
}

impl<T> Triangle<T> {
    pub fn new(a: T, b: T, c: T) -> Triangle<T> {
        Triangle { a, b, c }
    }

    pub fn a(&self) -> &T {
        &self.a
    }

    pub fn b(&self) -> &T {
        &self.b
    }

    pub fn c(&self) -> &T {
        &self.c
    }
}

pub struct TriangleIterator<'a, T> {
    t: &'a Triangle<T>,
    pos: u32,
}

impl<'a, T> Iterator for TriangleIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = match self.pos {
            0 => &self.t.a,
            1 => &self.t.b,
            2 => &self.t.c,
            _ => return None,
        };
        self.pos += 1;
        Some(ret)
    }
}
impl<'a, T> IntoIterator for &'a Triangle<T> {
    type Item = &'a T;
    type IntoIter = TriangleIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        TriangleIterator { t: self, pos: 0 }
    }
}

// impl<T> Triangle<Vector2<T>>
// where
//     T: nalgebra::Scalar
//         + num_traits::AsPrimitive<f32>
//         + num_traits::identities::Zero
//         + nalgebra::ClosedSub
//         + nalgebra::ClosedAdd
//         + nalgebra::ClosedMul,
// {
//     pub fn barycentric(&self, p: Vector2<T>) -> Triangle<f32> {
//         // https://gamedev.stackexchange.com/questions/23743/whats-the-most-efficient-way-to-find-barycentric-coordinates
//         let v0 = self.b - self.a;
//         let v1 = self.c - self.a;
//         let v2 = p - self.a;
//         let d00 = v0.dot(&v0);
//         let d01 = v0.dot(&v1);
//         let d11 = v1.dot(&v1);
//         let d20 = v2.dot(&v0);
//         let d21 = v2.dot(&v1);
//         let inverse_denom = 1.0 / ((d00 * d11) - (d01 * d01)).as_();

//         let j = (d11 * d20 - d01 * d21).as_() * inverse_denom;
//         let k = (d00 * d21 - d01 * d20).as_() * inverse_denom;
//         let i = 1.0f32 - j - k;

//         Triangle::new(i, j, k)
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    // fn barycentric_coordinates_test() {
    //     let t: Triangle<Vector2<i32>> = Triangle::new(vec2(0, 0), vec2(2, 0), vec2(2, 2));
    //     assert_eq!(t.barycentric(vec2(0, 0)), Triangle::new(1., 0., 0.));
    //     assert_eq!(t.barycentric(vec2(2, 0)), Triangle::new(0., 1., 0.));
    //     assert_eq!(t.barycentric(vec2(2, 2)), Triangle::new(0., 0., 1.));
    //     assert_eq!(t.barycentric(vec2(1, 1)), Triangle::new(0.5, 0.0, 0.5));
    //     assert_eq!(t.barycentric(vec2(1, 0)), Triangle::new(0.5, 0.5, 0.0));
    // }
}
