// Adapted from original author:
// https://github.com/svenstaro/bvh/blob/master/src/bvh/bvh_impl.rs

use crate::{
    aabb::AABB,
    hittable::{HitRecord, Hittable},
    ray::Ray,
};
use std::{cmp::Ordering, ops::Range};

#[derive(Debug, Clone)]
enum BvhNode {
    Leaf {
        shape_index: usize,
    },
    Node {
        left_index: usize,
        left_bbox: AABB,
        right_index: usize,
        right_bbox: AABB,
    },
}

#[derive(Clone)]
pub struct Bvh<T: Hittable> {
    nodes: Vec<BvhNode>,
    objects: Vec<T>,
}

impl<T: Hittable> Bvh<T> {
    pub fn new(objects: Vec<T>) -> Self {
        let indices = (0..objects.len()).collect::<Vec<usize>>();
        let expected_node_count = objects.len() * 2;
        let mut nodes = Vec::with_capacity(expected_node_count);
        BvhNode::build(&objects, &indices, &mut nodes);
        Bvh { nodes, objects }
    }
}

impl BvhNode {
    fn build(shapes: &[impl Hittable], indices: &[usize], nodes: &mut Vec<BvhNode>) -> usize {
        // If there is only one element left, don't split anymore
        if indices.len() == 1 {
            let shape_index = indices[0];
            let node_index = nodes.len();
            nodes.push(BvhNode::Leaf { shape_index });
            return node_index;
        }

        // Helper function to accumulate the AABB joint and the centroids AABB
        fn grow_convex_hull(convex_hull: (AABB, AABB), shape_aabb: &AABB) -> (AABB, AABB) {
            let center = shape_aabb.center();
            let convex_hull_aabbs = &convex_hull.0;
            let convex_hull_centroids = &convex_hull.1;
            (
                convex_hull_aabbs.merge(shape_aabb),
                convex_hull_centroids.grow(center),
            )
        }

        let mut convex_hull = Default::default();
        for index in indices {
            convex_hull = grow_convex_hull(convex_hull, shapes[*index].bounding_box());
        }
        let (aabb_bounds, centroid_bounds) = convex_hull;

        // From here on we handle the recursive case. This dummy is required,
        // because it's easier to update one parent node than the child nodes.
        let node_index = nodes.len();
        nodes.push(BvhNode::create_dummy());

        // Find the axis along which the shapes are spread the most.
        let split_axis = centroid_bounds.largest_axis();
        let split_axis_size = centroid_bounds.max[split_axis] - centroid_bounds.min[split_axis];

        // The following `if` partitions `indices` for recursively calling `Bvh::build`.
        let (child_l_index, child_l_aabb, child_r_index, child_r_aabb) = if split_axis_size
            < f64::EPSILON
        {
            // In this branch the shapes lie too close together so that splitting them in a
            // sensible way is not possible. Instead we just split the list of shapes in half.
            let (child_l_indices, child_r_indices) = indices.split_at(indices.len() / 2);
            let child_l_aabb = joint_aabb_of_shapes(child_l_indices, shapes);
            let child_r_aabb = joint_aabb_of_shapes(child_r_indices, shapes);

            // Proceed recursively.
            let child_l_index = BvhNode::build(shapes, child_l_indices, nodes);
            let child_r_index = BvhNode::build(shapes, child_r_indices, nodes);
            (child_l_index, child_l_aabb, child_r_index, child_r_aabb)
        } else {
            // Create six `Bucket`s, and six index assignment vector.
            const NUM_BUCKETS: usize = 6;
            let mut buckets: [Bucket; NUM_BUCKETS] = Default::default();
            let mut bucket_assignments: [Vec<usize>; NUM_BUCKETS] = Default::default();

            // In this branch the `split_axis_size` is large enough to perform meaningful splits.
            // We start by assigning the shapes to `Bucket`s.
            for idx in indices {
                let shape = &shapes[*idx];
                let shape_aabb = shape.bounding_box();
                let shape_center = shape_aabb.center();

                // Get the relative position of the shape centroid `[0.0..1.0]`.
                let bucket_num_relative =
                    (shape_center[split_axis] - centroid_bounds.min[split_axis]) / split_axis_size;

                // Convert that to the actual `Bucket` number.
                let bucket_num = (bucket_num_relative * NUM_BUCKETS as f64 - 0.01) as usize;

                // Extend the selected `Bucket` and add the index to the actual bucket.
                buckets[bucket_num].add_aabb(shape_aabb);
                bucket_assignments[bucket_num].push(*idx);
            }

            // Compute the costs for each configuration and select the best configuration.
            let (min_bucket, _min_cost, child_l_aabb, child_r_aabb) = (0..(NUM_BUCKETS - 1))
                .map(|i| {
                    let (l_buckets, r_buckets) = buckets.split_at(i + 1);
                    let child_l = l_buckets.iter().fold(Bucket::empty(), Bucket::join_bucket);
                    let child_r = r_buckets.iter().fold(Bucket::empty(), Bucket::join_bucket);

                    let cost = (child_l.size as f64 * child_l.aabb.surface_area()
                        + child_r.size as f64 * child_r.aabb.surface_area())
                        / aabb_bounds.surface_area();

                    (i, cost, child_l.aabb, child_r.aabb)
                })
                .min_by(|(_, cost1, _, _), (_, cost2, _, _)| {
                    cost1.partial_cmp(cost2).unwrap_or(Ordering::Equal)
                })
                .unwrap_or((0, f64::INFINITY, AABB::empty(), AABB::empty()));

            // Join together all index buckets.
            let (l_assignments, r_assignments) = bucket_assignments.split_at_mut(min_bucket + 1);
            let child_l_indices = concatenate_vectors(l_assignments);
            let child_r_indices = concatenate_vectors(r_assignments);

            // Proceed recursively.
            let child_l_index = BvhNode::build(shapes, &child_l_indices, nodes);
            let child_r_index = BvhNode::build(shapes, &child_r_indices, nodes);
            (child_l_index, child_l_aabb, child_r_index, child_r_aabb)
        };

        // Construct the actual data structure and replace the dummy node.
        debug_assert!(!child_l_aabb.is_empty());
        debug_assert!(!child_r_aabb.is_empty());
        nodes[node_index] = BvhNode::Node {
            left_bbox: child_l_aabb,
            left_index: child_l_index,
            right_bbox: child_r_aabb,
            right_index: child_r_index,
        };

        node_index
    }

    fn create_dummy() -> BvhNode {
        BvhNode::Leaf { shape_index: 0 }
    }

    fn traverse<'a>(
        nodes: &Vec<BvhNode>,
        node_index: usize,
        ray: &Ray,
        interval: &Range<f64>,
        shapes: &'a [impl Hittable],
    ) -> Option<HitRecord<'a>> {
        match &nodes[node_index] {
            BvhNode::Node {
                left_index,
                left_bbox,
                right_index,
                right_bbox,
            } => {
                let mut hit = None;
                if left_bbox.hit(ray, interval) {
                    hit = BvhNode::traverse(nodes, *left_index, ray, interval, shapes);
                }
                if right_bbox.hit(ray, interval) {
                    hit = BvhNode::traverse(
                        nodes,
                        *right_index,
                        ray,
                        &(interval.start..hit.as_ref().map_or(interval.end, |hit| hit.t)),
                        shapes,
                    )
                    .or(hit);
                }
                hit
            }
            BvhNode::Leaf { shape_index, .. } => shapes[*shape_index].hit(ray, interval),
        }
    }
}

impl<T: Hittable + Clone> Hittable for Bvh<T> {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord> {
        BvhNode::traverse(&self.nodes, 0, ray, interval, &self.objects)
    }

    fn bounding_box(&self) -> &AABB {
        unimplemented!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Bucket {
    /// The number of shapes in this `Bucket`.
    pub size: usize,

    /// The joint [`Aabb`] of the shapes in this [`Bucket`].
    pub aabb: AABB,
}

impl Bucket {
    /// Returns an empty bucket.
    pub fn empty() -> Bucket {
        Bucket {
            size: 0,
            aabb: AABB::default(),
        }
    }

    /// Extend this [`Bucket`] by a shape with the given [`Aabb`].
    pub fn add_aabb(&mut self, aabb: &AABB) {
        self.size += 1;
        self.aabb = self.aabb.merge(aabb);
    }

    /// Join the contents of two [`Bucket`]'s.
    pub fn join_bucket(a: Bucket, b: &Bucket) -> Bucket {
        Bucket {
            size: a.size + b.size,
            aabb: a.aabb.merge(&b.aabb),
        }
    }
}

pub fn concatenate_vectors<T: Sized>(vectors: &mut [Vec<T>]) -> Vec<T> {
    vectors.iter_mut().flat_map(|v| v.drain(..)).collect()
}

pub fn joint_aabb_of_shapes<T>(indices: &[usize], shapes: &[T]) -> AABB
where
    T: Hittable,
{
    indices.iter().fold(AABB::empty(), |aabb, index| {
        let shape = &shapes[*index];
        aabb.merge(shape.bounding_box())
    })
}
