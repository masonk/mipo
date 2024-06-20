use anyhow::{anyhow, Result};
use image::{io::Reader, ImageBuffer, Luma};
use nalgebra::Vector2;
use std::path::Path;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Label(u32);

type Triangle<T> = (Vector2<T>, Vector2<T>, Vector2<T>);
type Heightmap = ImageBuffer<Luma<u16>, Vec<u16>>;
type Coords = Vector2<u32>;

// All the data that can processed offline for a heightmap. Includes an error map
// of the rtin hierarchy.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RtinData {
    grid_side_length: u32,
    errors: Vec<f32>,
}
pub struct Options {
    _error_threshold: f32,
}
impl Default for Options {
    fn default() -> Options {
        Options {
            _error_threshold: 0.0,
        }
    }
}
pub struct Vertices;

pub fn preprocess_heightmap_from_img_path<P: AsRef<Path>>(path: P) -> Result<RtinData> {
    let img: Heightmap = Reader::open(path.as_ref())?.decode()?.into_luma16();
    preprocess_heightmap_from_img(&img)
}

pub fn preprocess_heightmap_from_img_bytes(img_buffer: &[u8]) -> Result<RtinData> {
    let img: Heightmap = Reader::new(std::io::Cursor::new(img_buffer))
        .with_guessed_format()?
        .decode()?
        .into_luma16();

    preprocess_heightmap_from_img(&img)
}

pub fn preprocess_heightmap_from_img(img: &Heightmap) -> Result<RtinData> {
    preprocess_heightmap(&img)
}

// For the smallest possible RTIN representation, with a dense configuration of triangles
// Calculate the error of the covering triangle.
pub fn preprocess_heightmap(heightmap: &Heightmap) -> Result<RtinData> {
    let (x, y) = heightmap.dimensions();
    if x != y {
        return Err(anyhow!(
            "A square heightmap is required, got {} x {}.",
            x,
            y
        ));
    }
    if !(x - 1).is_power_of_two() {
        return Err(anyhow!("rtin only works when the dimensions of the heightmap are 2^k + 1 x 2^k + 1 for some integer k. Got: {} x {}", x, y));
    }
    // "If the original input was a 2^k + 1 x 2^k + 1 array, the binary tree representing the hierarchy
    // is the complete binary tree of depth 2k + 1"
    // side = 2^k + 1
    // side - 1 = 2^k
    // log2(side - 1) = k
    let k = (x - 1).ilog2();
    let d = 2 * k + 1;
    // A complete binary tree of depth d contains 2^d - 1 nodes.
    // We are going to compute an error for every triangle in the hierarchy of rtin approximations
    let error_len: u32 = 2u32.pow(d) - 1;
    let mut errors: Vec<f32> = Vec::with_capacity(error_len as usize);

    // println!("side: {x}, k: {k}, d: {d}, error_len: {error_len}");

    /*
       Choice of error measure isn't specified by the Evans paper.

           "In our implementation, this is the maximum over points 'covered' by the triangle of the vertical
           distance from the point to the triangle."

       Other possible error measures could include: average distance between covered points and triangle,
       RMS of distance between covered points and triangle, etc. It likely does not matter a lot, so let's
       do the same as the paper: the error of a triangle is the maximum pointwise error between the height
       of a the rtin trinagle at a lattice point and the heightmap's true value for the height at that point.

       For our purposes, a point is 'covered' by the triangle if it is inside the triangle's projection
       onto the xy plane. The height of the triangle at a point p is the  interpolation of the
       height of the triangle's three vertices at that point (via barymetric coordinates).

    */
    for i in 0..error_len {
        if i == 0 {
            // i == 0 is the "root of the rtin hierarchy".
            // Unlike all other nodes, this node logically corresponds to a square - the original, unpartioned square.
            // Most rtin algorithms, including mesh extraction, can't do anything with the root. Its error is undefined.
            errors.push(0.0);
            continue;
        }
        let label = idx_to_label(i);
        let coords = coords(label, x);

        let a = vec2(coords.0[0] as f32, coords.0[1] as f32);
        let b = vec2(coords.1[0] as f32, coords.1[1] as f32);
        let c = vec2(coords.2[0] as f32, coords.2[1] as f32);

        let v0 = b - a;
        let v1 = c - a;

        let d00 = v0.dot(&v0);
        let d01 = v0.dot(&v1);
        let d11 = v1.dot(&v1);

        let inverse_denom = 1.0 / ((d00 * d11) - (d01 * d01)) as f32;

        let mut max: f32 = 0.0;
        // println!("{a:?}, {b:?}, {c:?}");

        for p in points_in_bounding_box((a, b, c)) {
            let v2 = p - a;
            let d20 = v2.dot(&v0);
            let d21 = v2.dot(&v1);

            let j = (d11 * d20 - d01 * d21) as f32 * inverse_denom;
            // Any point with a negative barymetric coordinate lies outside
            // of the triangle.
            if j < 0.0 {
                continue;
            }
            let k = (d00 * d21 - d01 * d20) as f32 * inverse_denom;
            if k < 0.0 {
                continue;
            }
            let i = 1.0f32 - j - k;
            if i < 0.0 {
                continue;
            }

            let a_z = heightmap.get_pixel(a[0] as u32, a[1] as u32)[0] as f32;
            let b_z = heightmap.get_pixel(b[0] as u32, b[1] as u32)[0] as f32;
            let c_z = heightmap.get_pixel(c[0] as u32, c[1] as u32)[0] as f32;
            let interpolated = a_z * i + b_z * j + c_z as f32 * k;
            let true_height = heightmap.get_pixel(p[0] as u32, p[1] as u32)[0] as f32;
            let error = (interpolated - true_height).abs();
            // println!("i: {i:.03}, j: {j:.03}, k: {k:03}, error: {error}, current max: {max}, true: {true_height}, interpolated: {interpolated}");
            max = error.max(max);
        }

        errors.push(max);
    }

    Ok(RtinData {
        grid_side_length: x,
        errors,
    })
}

fn _barycentric_coordinates(p: Vector2<i32>, (a, b, c): Triangle<i32>) -> (f32, f32, f32) {
    // https://gamedev.stackexchange.com/questions/23743/whats-the-most-efficient-way-to-find-barycentric-coordinates
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = v0.dot(&v0);
    let d01 = v0.dot(&v1);
    let d11 = v1.dot(&v1);
    let d20 = v2.dot(&v0);
    let d21 = v2.dot(&v1);
    let inverse_denom = 1.0 / ((d00 * d11) - (d01 * d01)) as f32;

    let j = (d11 * d20 - d01 * d21) as f32 * inverse_denom;
    let k = (d00 * d21 - d01 * d20) as f32 * inverse_denom;
    let i = 1.0f32 - j - k;

    (i, j, k)
}
fn points_in_bounding_box((a, b, c): Triangle<f32>) -> Vec<Vector2<f32>> {
    let mut points = vec![];

    let bottom_left = vec2(a[0].min(b[0]).min(c[0]), a[1].min(b[1]).min(c[1]));
    let top_right = vec2(a[0].max(b[0]).max(c[0]), a[1].max(b[1]).max(c[1]));

    for i in bottom_left[0] as u32..=top_right[0] as u32 {
        for j in bottom_left[1] as u32..=top_right[1] as u32 {
            points.push(vec2(i as f32, j as f32));
        }
    }
    points
}

// fn lattice_points_in_triangle((a, b, c): Triangle<i32>) -> Vec<Vector2<i32>> {
//     let mut points = vec![];

//     let bottom_left = vec2(a[0].min(b[0]).min(c[0]), a[1].min(b[1]).min(c[1]));
//     let top_right = vec2(a[0].max(b[0]).max(c[0]), a[1].max(b[1]).max(c[1]));

//     for i in bottom_left[0]..=top_right[0] {
//         for j in bottom_left[1]..=top_right[1] {
//             let p = vec2(i, j);
//             if point_in_triangle(p, (a, b, c)) {
//                 points.push(p);
//             }
//         }
//     }
//     points
// }

// fn point_in_triangle(
//     p: Vector2<i32>,
//     (a, b, c): (Vector2<i32>, Vector2<i32>, Vector2<i32>),
// ) -> bool {
//     // A point is in or on the edge of a triangle abc if it is to the same side of the each of the
//     // lines ab, bc, and ca. We find out which side the point is on by taking the "2d cross product"
//     // of ab x ap, bc x bp, and ca x cp respectively. We do this by noticing that adding a 0 third dimension
//     // gives us one term nonzero component to check.
//     // (x1, y1, 0) x (x2, y2, 0) = (0, 0, x1y2 - x2y1)
//     let ab = b - a;
//     let ap = p - a;
//     let sign_1 = ((ab[0] * ap[1]) - (ab[1] * ap[0])).signum();

//     let bc = c - b;
//     let bp = p - b;
//     let sign_2 = ((bc[0] * bp[1]) - (bc[1] * bp[0])).signum();
//     if sign_1 != 0 && sign_2 != 0 && sign_1 != sign_2 {
//         return false;
//     }

//     let ca = a - c;
//     let cp = p - c;
//     let sign_3 = ((ca[0] * cp[1]) - (ca[1] * cp[0])).signum();

//     if sign_1 != 0 && sign_3 != 0 && sign_1 != sign_3 {
//         return false;
//     }
//     if sign_2 != 0 && sign_3 != 0 && sign_2 != sign_3 {
//         return false;
//     }
//     return true;
// }
/*
From the original paper, each node in the rtin btree is assigned a "label" which traces the path from root to the node.

   "The label of a region is a description of the path from the root to its corresponding node in the binary tree. The path
   descriptiopn is the conacatenation of the labels of the dges on path from the root to the node, where an edge is labeled
   0 if it leads to a left child and 1 if it leads to a right child."

To deal with the problem of leading 0s, the BinId has a 1 added in front. If the tree is 6 levels deep, then the labels
of the leafs all have a 1 in the 7th bit, which is the MSB of the labels. The root node is therefore "1".
*/
fn idx_to_label(idx: u32) -> Label {
    let d = idx_depth(idx);

    // How many indices are used by nodes at higher depths than our node?
    // These are all accounted for by the msb of the label.
    let offset: u32 = 2u32.pow(d - 1) - 1;
    Label((1 << d - 1) + (idx - offset))
}

fn label_to_idx(Label(val): Label) -> u32 {
    let d = 32 - val.leading_zeros();
    let offset = 2u32.pow(d - 1) - 1;
    let mask = 1 << d - 1;
    let masked = val ^ mask;
    return masked + offset;
}

fn idx_depth(idx: u32) -> u32 {
    // What is the depth 'd' of node with index idx?
    // Depths are "1-indexed", i.e. the root node with idx 0 is depth 1, rather than depth 0.
    // We are looking for the greatest integer d such that 2^d - 1 < idx + 1
    let mut d = 1;
    while 2u32.pow(d) - 1 < idx + 1 {
        d += 1;
    }
    d
}

// Given the indice into the bintree of a node, return the indices of its two children, if they exist.
pub fn child_indexes(i: u32, grid_size: u32) -> Option<(u32, u32)> {
    if i >= grid_size / 2 {
        return None;
    }
    let Label(p) = idx_to_label(i);
    let base = p << 1;

    Some((label_to_idx(Label(base)), label_to_idx(Label(base + 1))))
}

fn vec2<T>(a: T, b: T) -> Vector2<T> {
    Vector2::new(a, b)
}

#[derive(Eq, PartialEq, Debug)]
enum Step {
    TopRight,
    BottomLeft,
    Left,
    Right,
}
fn coords(label: Label, grid_size: u32) -> Triangle<u32> {
    /*
     *   "Determining the coordinates of the three vertices of a triangle from its label is straightfoward. The label
     *    describes a path in the binary tree representing the surface. At each step in this path, as one descends
     *    from the root, one can construct the vertices of the left or right child triangle from the vertices of the
     *    parent. If (v1, v2, v3) are the vertices of the parent triangle, (vertices are listed in counter-clockwise
     *    order with v3 the right-angled vertex), then the left child is (v3, v1, m), and the right child is
     *    (v2, v3, m), where (mx, my) are the x,y coordinates of the midpoint of the line segment v1v2, and the z
     *    coordinate is obtained from the heightmap at m."
     *   "
     *
     *   To calculate the coords of a label, we reverse the process. We start from the root and apply each partitioning
     *   step encoded by a bit of the label.
     *
     *   grid size is always 2^k + 1 for some integer k. This ensures that the two base triangles are partitionable k times,
     *   and that the coordinates of the midpoint of the hypoteneuse are integers.
     *   a  .  .  .  .
     *   .  .
     *   .     .
     *   .        .
     *   .
     *   c  .  .  .  b
     */
    let mut a: Coords = Vector2::default();
    let mut b: Coords = Vector2::default();
    let mut c: Coords = Vector2::default();

    let steps = steps(label);
    use Step::*;

    for step in steps {
        match step {
            BottomLeft => {
                a[0] = grid_size - 1;
                a[1] = grid_size - 1;
                b[0] = 0;
                b[1] = 0;
                c[0] = 0;
                c[1] = grid_size - 1;
            }
            TopRight => {
                a[0] = 0;
                a[1] = 0;
                b[0] = grid_size - 1;
                b[1] = grid_size - 1;
                c[0] = grid_size - 1;
                c[1] = 0;
            }
            Left => {
                let (ap, bp, cp) = (c, a, (a + b) / 2);
                a = ap;
                b = bp;
                c = cp;
            }
            Right => {
                let (ap, bp, cp) = (b, c, (a + b) / 2);
                a = ap;
                b = bp;
                c = cp;
            }
        }
    }

    (a, b, c)
}

fn steps(Label(mut id): Label) -> Vec<Step> {
    use Step::*;
    // The root is a square, it has no partitioning. Most rtin algorithms don't operate on root.
    if id == 1 {
        return Vec::new();
    }
    let mut steps = Vec::new();
    loop {
        let lsb = id & 1;
        id = id >> 1;

        if id <= 1 {
            if lsb == 0 {
                steps.push(BottomLeft)
            } else {
                steps.push(TopRight)
            }
            break;
        } else {
            if lsb == 0 {
                steps.push(Left)
            } else {
                steps.push(Right)
            }
        }
    }

    steps.reverse();
    steps
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_idx_depth() {
        assert_eq!(idx_depth(0), 1);
        assert_eq!(idx_depth(1), 2);
        assert_eq!(idx_depth(2), 2);
        assert_eq!(idx_depth(3), 3);
        assert_eq!(idx_depth(4), 3);
        assert_eq!(idx_depth(5), 3);
        assert_eq!(idx_depth(6), 3);
        assert_eq!(idx_depth(7), 4);
        assert_eq!(idx_depth(14), 4);
        assert_eq!(idx_depth(15), 5);
        assert_eq!(idx_depth(16), 5);
    }

    #[test]
    fn test_label() {
        assert_eq!(idx_to_label(0), Label(0b1));
        assert_eq!(idx_to_label(1), Label(0b10));
        assert_eq!(idx_to_label(2), Label(0b11));
        assert_eq!(idx_to_label(3), Label(0b100));
        assert_eq!(idx_to_label(4), Label(0b101));
        assert_eq!(idx_to_label(5), Label(0b110));
        assert_eq!(idx_to_label(6), Label(0b111));
        assert_eq!(idx_to_label(7), Label(0b1000));
        assert_eq!(idx_to_label(14), Label(0b1111));
        assert_eq!(idx_to_label(15), Label(0b10000));
        assert_eq!(idx_to_label(16), Label(0b10001));
    }

    #[test]
    fn test_steps() {
        use super::Step::*;
        assert_eq!(steps(Label(0b10110)), vec![BottomLeft, Right, Right, Left]);
        assert_eq!(steps(Label(0b11)), vec![TopRight]);
        assert_eq!(steps(Label(0b110)), vec![TopRight, Left]);
        assert_eq!(
            steps(Label(0b1011011100010)),
            vec![
                BottomLeft, Right, Right, Left, Right, Right, Right, Left, Left, Left, Right, Left
            ]
        );
    }

    #[test]
    fn label_to_idx_test() {
        for i in 0..10000 {
            let label: Label = idx_to_label(i);
            let actual = label_to_idx(label);
            assert_eq!(
                label_to_idx(label),
                i,
                "{i} -> {actual} (should be same). Label was: {label:?}"
            );
        }
    }

    #[test]
    fn child_indexes_test() {
        match child_indexes(0, 9) {
            Some((l, r)) => {
                assert_eq!(idx_to_label(l), Label(0b10));
                assert_eq!(idx_to_label(r), Label(0b11));
            }
            None => {
                assert!(false, "0, 9 should have children, got None")
            }
        }
        match child_indexes(1, 9) {
            Some((l, r)) => {
                assert_eq!(idx_to_label(l), Label(0b100));
                assert_eq!(idx_to_label(r), Label(0b101));
            }
            None => {
                assert!(false, "1, 9 should have children, got None")
            }
        }
        match child_indexes(2, 9) {
            Some((l, r)) => {
                assert_eq!(idx_to_label(l), Label(0b110));
                assert_eq!(idx_to_label(r), Label(0b111));
            }
            None => {
                assert!(false, "2, 9 should have children, got None")
            }
        }
        match child_indexes(2, 5) {
            Some((_, _)) => {
                assert!(false, "2, 5 should have no children")
            }
            None => {}
        }
    }

    #[test]
    fn test_coords() {
        assert_eq!(coords(Label(0b10), 5), (vec2(4, 4), vec2(0, 0), vec2(0, 4)));
        assert_eq!(coords(Label(0b11), 5), (vec2(0, 0), vec2(4, 4), vec2(4, 0)));
        assert_eq!(
            coords(Label(0b1010), 5),
            (vec2(2, 2), vec2(0, 0), vec2(0, 2))
        );
    }

    // #[test]
    // fn points_in_triangle() {
    //     let points = points_in_projection((vec2(8, 0), vec(0, 8), vec(0, 0)));
    // }
    // #[test]
    // fn point_in_triangle_test() {
    //     let triangle = (vec2(0, 0), vec2(20, 0), vec2(0, 20));

    //     assert!(point_in_triangle(vec2(5, 5), triangle));
    //     assert!(point_in_triangle(vec2(20, 0), triangle));
    //     assert!(point_in_triangle(vec2(0, 0), triangle));
    //     assert!(point_in_triangle(vec2(0, 20), triangle));

    //     assert!(
    //         !point_in_triangle(vec2(20, 1), triangle),
    //         "(20, 1) is outside"
    //     );
    //     assert!(
    //         !point_in_triangle(vec2(0, -1), triangle),
    //         "(0, -1) is outside"
    //     );
    //     assert!(
    //         !point_in_triangle(vec2(0, 21), triangle),
    //         "(0, 21) is outside"
    //     );
    // }

    #[test]
    fn barycentric_coordinates_test() {
        let t: Triangle<i32> = (vec2(0, 0), vec2(2, 0), vec2(2, 2));
        assert_eq!(barycentric_coordinates(vec2(0, 0), t), (1., 0., 0.));
        assert_eq!(barycentric_coordinates(vec2(2, 0), t), (0., 1., 0.));
        assert_eq!(barycentric_coordinates(vec2(2, 2), t), (0., 0., 1.));
        assert_eq!(barycentric_coordinates(vec2(1, 1), t), (0.5, 0.0, 0.5));
        assert_eq!(barycentric_coordinates(vec2(1, 0), t), (0.5, 0.5, 0.0));
    }

    #[test]
    fn preprocess_heightmap_test() {
        let heightmap = Heightmap::from_vec(
            9,
            9,
            vec![
                767, 991, 704, 615, 399, 6, 554, 544, 770, 785, 170, 154, 470, 27, 670, 291, 828,
                928, 875, 117, 950, 592, 901, 36, 470, 537, 994, 74, 792, 403, 987, 676, 182, 130,
                887, 552, 45, 273, 665, 983, 845, 299, 59, 650, 765, 712, 309, 412, 840, 197, 396,
                90, 178, 396, 799, 415, 665, 421, 80, 14, 498, 781, 383, 820, 632, 877, 651, 101,
                532, 674, 587, 464, 95, 959, 691, 778, 563, 405, 826, 340, 109,
            ],
        )
        .unwrap();
        let rtin = preprocess_heightmap(&heightmap).unwrap();
        assert_eq!(
            rtin.errors,
            vec![
                0.0, 862.25, 762.875, 862.25, 641.0, 771.125, 644.25, 747.0, 624.0, 616.5, 737.5,
                678.75, 616.5, 746.0, 746.0, 624.0, 747.0, 490.0, 624.0, 338.0, 688.5, 737.5,
                338.0, 510.0, 485.75, 688.5, 483.5, 746.0, 453.5, 269.5, 746.0, 220.5, 265.5,
                566.0, 199.0, 289.5, 283.5, 275.5, 275.0, 404.5, 404.5, 795.5, 795.5, 317.0, 373.0,
                253.0, 253.0, 649.5, 649.5, 221.0, 470.5, 673.0, 688.5, 333.5, 623.0, 453.5, 275.5,
                340.5, 453.5, 327.5, 208.0, 475.5, 269.5, 48.5, 220.5, 265.5, 48.5, 199.0, 566.0,
                151.0, 199.0, 12.0, 289.5, 127.5, 12.0, 275.0, 265.5, 220.5, 275.0, 404.5, 82.0,
                228.0, 404.5, 795.5, 36.0, 386.0, 795.5, 317.0, 290.0, 373.0, 317.0, 253.0, 228.0,
                82.0, 253.0, 649.5, 623.0, 197.0, 649.5, 221.0, 118.0, 470.5, 221.0, 673.0, 63.5,
                255.5, 673.0, 333.5, 197.0, 623.0, 333.5, 188.5, 238.0, 153.0, 188.5, 340.5, 218.0,
                178.0, 340.5, 195.0, 327.5, 46.0, 195.0, 134.5, 153.0, 238.0, 134.5,
            ]
        );
    }

    #[test]
    fn preprocess_grand_canyon_test() {
        let img: Heightmap = Reader::open("assets/grand_canyon_small_heightmap.png")
            .unwrap()
            .decode()
            .unwrap()
            .into_luma16();

        let rtin = preprocess_heightmap(&img).unwrap();
    }
}
