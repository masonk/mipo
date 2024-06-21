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
