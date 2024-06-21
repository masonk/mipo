use crate::geometry::{vec2, vec3, Triangle, Vector2, Vector3};

use anyhow::{anyhow, Result};
use image::{io::Reader, ImageBuffer, Luma};
#[allow(unused_imports)]
use log::{info, warn};

use std::collections::HashMap;
use std::path::Path;

#[cfg(feature = "serde")]
use ciborium;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Label(u32);

type Heightmap = ImageBuffer<Luma<u16>, Vec<u16>>;
type Coords = Vector2<u32>;

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RtinTriangle {
    pub error: f32,
    pub vertices: Triangle<Vector3<f32>>, // CCW ordering, last vertice is the right angle
}

// All the data that can be processed offline for a heightmap. Includes an error map
// of the rtin hierarchy.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RtinData {
    pub(crate) min_height: u16,
    pub(crate) max_height: u16,
    pub(crate) grid_size: u32,
    pub(crate) triangles: Vec<RtinTriangle>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MeshData {
    pub vertices: Vec<Vector3<f32>>, // domain [0, 1]
    pub indices: Vec<u32>,
}

pub fn threshold_triangle(
    threshold: f32,
    idx: u32,
    rtin_data: &RtinData,
    triangles: &mut Vec<u32>,
) {
    if idx as usize >= rtin_data.triangles.len() {
        return;
    }
    let err = rtin_data.triangles[idx as usize].error;
    if err <= threshold {
        triangles.push(idx);
        return;
    }

    let (l, r) = child_indexes(idx);
    if l as usize >= rtin_data.triangles.len() {
        triangles.push(idx);
        return;
    }
    threshold_triangle(threshold, l, rtin_data, triangles);
    threshold_triangle(threshold, r, rtin_data, triangles);
}

// Returns the indexes of the triangles of an rtin mesh where all triangles
// are either leafs or have an error below error_threshold
fn thresholded_triangles(error_threshold: f32, rtin_data: &RtinData) -> Vec<u32> {
    let mut triangles: Vec<u32> = vec![];
    // idx 0 is the "root node" of the tree, which is a quadrilateral and should
    // be ignored.
    threshold_triangle(error_threshold, 1, &rtin_data, &mut triangles);
    threshold_triangle(error_threshold, 2, &rtin_data, &mut triangles);

    triangles
}
// Construct a mesh from an rtin
pub fn thresholded_mesh_data(error_threshold: f32, rtin_data: &RtinData) -> MeshData {
    let mut indices: Vec<u32> = vec![];
    let mut vertices: Vec<Vector3<f32>> = vec![];
    let triangle_indices = thresholded_triangles(error_threshold, rtin_data);
    let mut vertice_lookup = HashMap::<u32, usize>::new();

    for idx in triangle_indices {
        let t = &rtin_data.triangles[idx as usize];
        for v in t.vertices.into_iter() {
            let v_id = v[1] as u32 * rtin_data.grid_size + v[0] as u32;

            let v_idx = if vertice_lookup.contains_key(&v_id) {
                *vertice_lookup.get(&v_id).unwrap()
            } else {
                let end = vertices.len();
                vertices.push(v.clone());
                vertice_lookup.insert(v_id, end);
                end
            };
            indices.push(v_idx as u32);
        }
    }

    MeshData { vertices, indices }
}

pub fn preprocess_heightmap_from_img_path<P: AsRef<Path>>(path: P) -> Result<RtinData> {
    let img: Heightmap = Reader::open(path.as_ref())?.decode()?.into_luma16();

    #[cfg(feature = "serde")]
    {
        // Check if there is a .rtin file next to the image. If so, deserialize and use it.
        let mut rtin_path = path.as_ref().to_path_buf();
        rtin_path.set_extension("rtin");
        use std::fs::OpenOptions;
        let mut options = OpenOptions::new();
        options.create(false).read(true).write(false);

        if let Ok(file) = options.open(rtin_path) {
            let reader = std::io::BufReader::new(file);
            if let Ok(rtin) = ciborium::from_reader(reader) {
                info!("Restored rtin preprocessed data from disc.");
                return Ok(rtin);
            } else {
                info!("Unable to restore rtin data from disc: data corrupt? Older version? Will recompute and clobber.");
            }
        } else {
            info!("Looked for serialized rtin data, but either it wasn't there or we couldn't read it.")
        }
    }

    let rtin = preprocess_heightmap_from_img(&img)?;

    #[cfg(feature = "serde")]
    {
        use std::fs::File;
        // Check if there is a .rtin file next to the image. If so, deserialize and use it.
        let mut rtin_path = path.as_ref().to_path_buf();
        rtin_path.set_extension("rtin");

        match File::create(rtin_path) {
            Ok(file) => {
                let writer = std::io::BufWriter::new(file);

                match ciborium::into_writer(&rtin, writer) {
                    Ok(_) => {
                        info!("Wrote rtin data disc.");
                    }
                    Err(e) => {
                        info!("{e}")
                    }
                }
            }
            Err(e) => {
                warn!("{e}");
            }
        }
    }

    Ok(rtin)
}

pub fn preprocess_heightmap_from_img(img: &Heightmap) -> Result<RtinData> {
    preprocess_heightmap(&img)
}

// How many triangles in the rtin hierarchy?
fn num_triangles(grid_size: u32) -> u32 {
    /*
     "If the original input was a 2^k + 1 x 2^k + 1 array, the binary tree representing the hierarchy
     is the complete binary tree of depth 2k + 1"
     side = 2^k + 1
     side - 1 = 2^k
     log2(side - 1) = k

    A complete binary tree of depth d contains 2^d nodes.
    */

    let k = (grid_size - 1).ilog2();
    let d = 2 * k + 1;
    2u32.pow(d)
}

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

    let mut min_height: u16 = 0;
    let mut max_height: u16 = 0;
    let num_triangles = num_triangles(x);
    let mut errors: Vec<f32> = Vec::with_capacity(num_triangles as usize);
    let mut triangles: Vec<RtinTriangle> = Vec::with_capacity(num_triangles as usize);
    // let mut heights: Vec<Vector3<u16>> = Vec::with_capacity(num_triangles as usize);

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
       height of the triangle's three vertices at that point (via barycentric coordinates).

    */
    for i in 0..num_triangles {
        if i == 0 {
            /*
             i == 0 is the "root of the rtin hierarchy".
             Unlike all other nodes, this node logically corresponds to a square - the original, unpartioned square.
             Most rtin algorithms, including mesh extraction, can't do anything with the root. Its error is undefined.
            */
            errors.push(0.0);
            triangles.push(RtinTriangle::default());
            // heights.push(vec3(0, 0, 0));
            continue;
        }
        let label = idx_to_label(i);
        let coords = coords(label, x);

        let a = vec2(coords.a[0] as f32, coords.a[1] as f32);
        let b = vec2(coords.b[0] as f32, coords.b[1] as f32);
        let c = vec2(coords.c[0] as f32, coords.c[1] as f32);

        let a_z = heightmap.get_pixel(a[0] as u32, a[1] as u32)[0];
        let b_z = heightmap.get_pixel(b[0] as u32, b[1] as u32)[0];
        let c_z = heightmap.get_pixel(c[0] as u32, c[1] as u32)[0];

        // heights.push(vec3(a_z, b_z, c_z));

        let a_zf = a_z as f32;
        let b_zf = b_z as f32;
        let c_zf = c_z as f32;

        let v0 = b - a;
        let v1 = c - a;

        let d00 = v0.dot(&v0);
        let d01 = v0.dot(&v1);
        let d11 = v1.dot(&v1);

        let inverse_denom = 1.0 / ((d00 * d11) - (d01 * d01)) as f32;

        let mut max: f32 = 0.0;
        // println!("{a:?}, {b:?}, {c:?}");

        for p in points_in_bounding_box(Triangle { a, b, c }) {
            let v2 = p - a;
            let d20 = v2.dot(&v0);
            let d21 = v2.dot(&v1);

            let j = (d11 * d20 - d01 * d21) as f32 * inverse_denom;
            // Any point with a negative barycentric coordinate lies outside
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

            let interpolated = a_zf * i + b_zf * j + c_zf as f32 * k;
            let true_height = heightmap.get_pixel(p[0] as u32, p[1] as u32)[0] as f32;
            let error = (interpolated - true_height).abs();
            // println!("i: {i:.03}, j: {j:.03}, k: {k:03}, error: {error}, current max: {max}, true: {true_height}, interpolated: {interpolated}");
            max = error.max(max);
        }

        for p in heightmap.pixels() {
            min_height = p[0].min(min_height);
            max_height = p[0].max(max_height);
        }

        errors.push(max);
        let triangle = RtinTriangle {
            error: max,
            vertices: Triangle::new(
                vec3(a[0], a[1], a_z as f32 / std::u16::MAX as f32),
                vec3(b[0], b[1], b_z as f32 / std::u16::MAX as f32),
                vec3(c[0], c[1], c_z as f32 / std::u16::MAX as f32),
            ),
        };
        triangles.push(triangle);
    }

    Ok(RtinData {
        grid_size: x,
        min_height,
        max_height,
        triangles,
    })
}

fn points_in_bounding_box(Triangle { a, b, c }: Triangle<Vector2<f32>>) -> Vec<Vector2<f32>> {
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

// Given the indice into the bintree of a node, return the indices that would correspond to its (left, right) children.
// Note: it is the callers responsibility to ensure that the children indices exist in the rtin hierarchy.
pub fn child_indexes(i: u32) -> (u32, u32) {
    let Label(p) = idx_to_label(i);
    let base = p << 1;

    (label_to_idx(Label(base)), label_to_idx(Label(base + 1)))
}

#[derive(Eq, PartialEq, Debug)]
enum Step {
    TopRight,
    BottomLeft,
    Left,
    Right,
}
fn coords(label: Label, grid_size: u32) -> Triangle<Vector2<u32>> {
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

    Triangle::new(a, b, c)
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
    use crate::geometry::{vec2, vec3};

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
    fn num_triangles_test() {
        assert_eq!(num_triangles(5), 32);
        assert_eq!(num_triangles(9), 128);
        assert_eq!(num_triangles(17), 512);
    }

    #[test]
    fn child_indexes_test() {
        {
            let (l, r) = child_indexes(0);
            assert_eq!(idx_to_label(l), Label(0b10));
            assert_eq!(idx_to_label(r), Label(0b11));
        }
        {
            let (l, r) = child_indexes(1);
            assert_eq!(idx_to_label(l), Label(0b100));
            assert_eq!(idx_to_label(r), Label(0b101));
        }
        {
            let (l, r) = child_indexes(2);
            assert_eq!(idx_to_label(l), Label(0b110));
            assert_eq!(idx_to_label(r), Label(0b111));
        }
    }

    #[test]
    fn test_coords() {
        assert_eq!(
            coords(Label(0b10), 5),
            Triangle::new(vec2(4, 4), vec2(0, 0), vec2(0, 4))
        );
        assert_eq!(
            coords(Label(0b11), 5),
            Triangle::new(vec2(0, 0), vec2(4, 4), vec2(4, 0))
        );
        assert_eq!(
            coords(Label(0b1010), 5),
            Triangle::new(vec2(2, 2), vec2(0, 0), vec2(0, 2))
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
    fn thresholded_mesh_data_test() {
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

        let mesh_data = thresholded_mesh_data(100.0, &rtin);
        assert!(mesh_data.indices.len() % 3 == 0);
        assert_eq!(
            mesh_data,
            MeshData {
                vertices: vec![
                    vec3(4.0, 6.0, 0.0012207218),
                    vec3(2.0, 6.0, 0.010147249),
                    vec3(3.0, 7.0, 0.0099336235),
                    vec3(4.0, 8.0, 0.00859083),
                    vec3(4.0, 4.0, 0.012893873),
                    vec3(3.0, 5.0, 0.012817578),
                    vec3(2.0, 8.0, 0.010543984),
                    vec3(1.0, 7.0, 0.009643702),
                    vec3(0.0, 8.0, 0.0014496071),
                    vec3(6.0, 8.0, 0.012603953),
                    vec3(6.0, 6.0, 0.007598993),
                    vec3(5.0, 7.0, 0.0081178),
                    vec3(8.0, 8.0, 0.0016632334),
                    vec3(7.0, 7.0, 0.008957046),
                    vec3(5.0, 5.0, 0.0060425727),
                    vec3(2.0, 4.0, 0.010147249),
                    vec3(2.0, 2.0, 0.014496071),
                    vec3(1.0, 3.0, 0.012085145),
                    vec3(0.0, 4.0, 0.000686656),
                    vec3(3.0, 3.0, 0.015060655),
                    vec3(0.0, 2.0, 0.013351644),
                    vec3(1.0, 1.0, 0.0025940337),
                    vec3(0.0, 0.0, 0.01170367),
                    vec3(0.0, 6.0, 0.012191959),
                    vec3(1.0, 5.0, 0.004715038),
                    vec3(4.0, 2.0, 0.0137483785),
                    vec3(6.0, 2.0, 0.0071717403),
                    vec3(5.0, 1.0, 0.010223545),
                    vec3(4.0, 0.0, 0.00608835),
                    vec3(5.0, 3.0, 0.002777142),
                    vec3(6.0, 0.0, 0.008453499),
                    vec3(7.0, 1.0, 0.01263447),
                    vec3(8.0, 0.0, 0.011749447),
                    vec3(2.0, 0.0, 0.010742351),
                    vec3(3.0, 1.0, 0.0071717403),
                    vec3(6.0, 4.0, 0.0009002823),
                    vec3(7.0, 5.0, 0.002716106),
                    vec3(8.0, 4.0, 0.0116731515),
                    vec3(8.0, 6.0, 0.0058442056),
                    vec3(8.0, 2.0, 0.015167468),
                    vec3(7.0, 3.0, 0.013534753)
                ],
                indices: vec![
                    0, 1, 2, 3, 0, 2, 0, 4, 5, 1, 0, 5, 6, 1, 7, 8, 6, 7, 6, 3, 2, 1, 6, 2, 9, 10,
                    11, 3, 9, 11, 9, 12, 13, 10, 9, 13, 0, 10, 14, 4, 0, 14, 0, 3, 11, 10, 0, 11,
                    15, 16, 17, 18, 15, 17, 15, 4, 19, 16, 15, 19, 20, 16, 21, 22, 20, 21, 20, 18,
                    17, 16, 20, 17, 23, 1, 24, 18, 23, 24, 23, 8, 7, 1, 23, 7, 15, 1, 5, 4, 15, 5,
                    15, 18, 24, 1, 15, 24, 25, 26, 27, 28, 25, 27, 25, 4, 29, 26, 25, 29, 30, 26,
                    31, 32, 30, 31, 30, 28, 27, 26, 30, 27, 33, 16, 34, 28, 33, 34, 33, 22, 21, 16,
                    33, 21, 25, 16, 19, 4, 25, 19, 25, 28, 34, 16, 25, 34, 35, 10, 36, 37, 35, 36,
                    35, 4, 14, 10, 35, 14, 38, 10, 13, 12, 38, 13, 38, 37, 36, 10, 38, 36, 39, 26,
                    40, 37, 39, 40, 39, 32, 31, 26, 39, 31, 35, 26, 29, 4, 35, 29, 35, 37, 40, 26,
                    35, 40
                ]
            }
        )
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
        let actual: Vec<f32> = rtin.triangles.iter().map(|t| t.error).collect();
        assert_eq!(
            actual,
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
                178.0, 340.5, 195.0, 327.5, 46.0, 195.0, 134.5, 153.0, 238.0, 134.5, 0.0
            ]
        );
        assert_eq!(rtin.triangles.len(), num_triangles(9) as usize);
    }

    #[test]
    fn preprocess_grand_canyon_test() {
        let img: Heightmap = Reader::open("assets/grand_canyon_small_heightmap.png")
            .unwrap()
            .decode()
            .unwrap()
            .into_luma16();

        let rtin = preprocess_heightmap(&img).unwrap();

        assert_eq!(
            rtin.triangles.len(),
            num_triangles(img.dimensions().0) as usize
        );
    }

    #[test]
    fn thresholded_triangles_grand_canyon_test() {
        let rtin_data =
            preprocess_heightmap_from_img_path("assets/grand_canyon_small_heightmap.png").unwrap();

        let num_leafs = num_triangles(rtin_data.grid_size) / 2;

        let triangles_0 = thresholded_triangles(0.0, &rtin_data);
        assert_eq!(triangles_0.len(), num_leafs as usize);

        let triangles_20000: Vec<u32> = thresholded_triangles(20000.0, &rtin_data);
        assert_eq!(
            triangles_20000,
            vec![
                63, 259, 1043, 1044, 522, 130, 32, 135, 136, 68, 139, 563, 1129, 1130, 282, 141,
                142, 17, 37, 155, 627, 628, 314, 78, 319, 320, 160, 161, 325, 326, 327, 328, 164,
                82, 83, 339, 681, 682, 170, 42, 87, 355, 356, 178, 179, 180, 90, 367, 737, 738,
                184, 185, 373, 374, 187, 377, 378, 94, 11, 51, 847, 848, 424, 425, 853, 854, 106,
                107, 217, 218, 54, 27, 57, 58, 119, 241, 242, 60, 61, 125, 126
            ]
        );

        let triangles_30000 = thresholded_triangles(30000.0, &rtin_data);
        assert_eq!(
            triangles_30000,
            vec![15, 33, 34, 8, 19, 20, 10, 11, 25, 26, 6]
        );
    }
}
