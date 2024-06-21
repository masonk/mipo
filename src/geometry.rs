pub trait Point = std::fmt::Debug + Default + PartialEq + Clone;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Triangle<T>
where
    T: Point,
{
    pub a: T,
    pub b: T,
    pub c: T,
}

impl<T> Triangle<T>
where
    T: Point,
{
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

pub struct TriangleIterator<'a, T>
where
    T: Point,
{
    t: &'a Triangle<T>,
    pos: u32,
}

impl<'a, T> Iterator for TriangleIterator<'a, T>
where
    T: Point,
{
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
impl<'a, T> IntoIterator for &'a Triangle<T>
where
    T: Point,
{
    type Item = &'a T;
    type IntoIter = TriangleIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        TriangleIterator { t: self, pos: 0 }
    }
}
