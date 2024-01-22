use crate::mesh::{GpuMeshIndex, GpuMeshes, GpuNode, GpuNodeBuffer};
use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::view::VisibilitySystems;
use bevy::render::{render_resource::*, RenderApp};
use bevy::render::{Extract, Render};
use bevy::transform::TransformSystem;
use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;
use bvh::bvh::BVH;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::marker::PhantomData;

pub struct InstancePlugin;
impl Plugin for InstancePlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<ExtractedInstances>()
                .init_resource::<InstanceRenderAssets>()
                .add_systems(Render, prepare_instances);
        }
    }
}

#[derive(Default)]
pub struct GenericInstancePlugin<M: Into<StandardMaterial>>(PhantomData<M>);
impl<M> Plugin for GenericInstancePlugin<M>
where
    M: Into<StandardMaterial> + Asset,
{
    fn build(&self, app: &mut App) {
        app.add_event::<InstanceEvent<M>>().add_systems(
            PostUpdate,
            instance_event_system::<M>
                .after(TransformSystem::TransformPropagate)
                .after(VisibilitySystems::VisibilityPropagate)
                .after(VisibilitySystems::CalculateBounds),
        );

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(ExtractSchedule, extract_instances::<M>);
        }
    }
}

#[derive(Default, Resource)]
pub struct InstanceRenderAssets {
    pub instance_buffer: StorageBuffer<GpuInstanceBuffer>,
    pub instance_node_buffer: StorageBuffer<GpuNodeBuffer>,
}

impl InstanceRenderAssets {
    pub fn set(&mut self, instances: Vec<GpuInstance>, instance_nodes: Vec<GpuNode>) {
        self.instance_buffer.get_mut().data = instances;

        self.instance_node_buffer.get_mut().count = instance_nodes.len() as u32;
        self.instance_node_buffer.get_mut().data = instance_nodes;
    }

    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.instance_buffer.write_buffer(device, queue);
        self.instance_node_buffer.write_buffer(device, queue);
    }
}

#[derive(Event)]
pub enum InstanceEvent<M: Into<StandardMaterial> + Asset> {
    Created(Entity, Handle<Mesh>, Handle<M>, Visibility),
    Modified(Entity, Handle<Mesh>, Handle<M>, Visibility),
    Removed(Entity),
}

#[allow(clippy::type_complexity)]
fn instance_event_system<M: Into<StandardMaterial> + Asset>(
    mut events: EventWriter<InstanceEvent<M>>,
    mut removed: RemovedComponents<Handle<Mesh>>,
    mut set: ParamSet<(
        Query<
            (Entity, &Handle<Mesh>, &Handle<M>, &Visibility),
            Or<(Added<Handle<Mesh>>, Added<Handle<M>>)>,
        >,
        Query<
            (Entity, &Handle<Mesh>, &Handle<M>, &Visibility),
            Or<(
                Changed<GlobalTransform>,
                Changed<Handle<Mesh>>,
                Changed<Handle<M>>,
                Changed<Visibility>,
            )>,
        >,
    )>,
) {
    for entity in removed.read() {
        events.send(InstanceEvent::Removed(entity));
    }
    for (entity, mesh, material, visibility) in &set.p0() {
        events.send(InstanceEvent::Created(
            entity,
            mesh.clone_weak(),
            material.clone_weak(),
            *visibility,
        ));
    }
    for (entity, mesh, material, visibility) in &set.p1() {
        events.send(InstanceEvent::Modified(
            entity,
            mesh.clone_weak(),
            material.clone_weak(),
            *visibility,
        ));
    }
}

#[allow(clippy::type_complexity)]
#[derive(Default, Resource)]
pub struct ExtractedInstances {
    extracted: Vec<(
        Entity,
        Aabb,
        GlobalTransform,
        Handle<Mesh>,
        UntypedHandle,
        Visibility,
    )>,
    removed: Vec<Entity>,
}

fn extract_instances<M: Into<StandardMaterial> + Asset>(
    mut events: Extract<EventReader<InstanceEvent<M>>>,
    query: Extract<Query<(&Aabb, &GlobalTransform)>>,
    mut extracted_instances: ResMut<ExtractedInstances>,
) {
    let mut extracted = vec![];
    let mut removed = vec![];

    for event in events.read() {
        match event {
            InstanceEvent::Created(entity, mesh, material, visibility)
            | InstanceEvent::Modified(entity, mesh, material, visibility) => {
                if let Ok((aabb, transform)) = query.get(*entity) {
                    extracted.push((
                        *entity,
                        *aabb,
                        *transform,
                        mesh.clone_weak(),
                        material.clone_weak().untyped(),
                        *visibility,
                    ));
                }
            }
            InstanceEvent::Removed(entity) => removed.push(*entity),
        }
    }

    extracted_instances.extracted.append(&mut extracted);
    extracted_instances.removed.append(&mut removed);
}

type Instances = BTreeMap<Entity, (GpuInstance, Visibility)>;

#[allow(clippy::too_many_arguments)]
fn prepare_instances(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut render_assets: ResMut<InstanceRenderAssets>,
    mut extracted_instances: ResMut<ExtractedInstances>,
    mut collection: Local<Instances>,
    meshes: Res<GpuMeshes>,
) {
    let instance_changed =
        !extracted_instances.extracted.is_empty() || !extracted_instances.removed.is_empty();

    for removed in extracted_instances.removed.drain(..) {
        collection.remove(&removed);
    }

    let mut prepare_next_frame = vec![];

    for (entity, aabb, transform, mesh, _material, visibility) in
        extracted_instances.extracted.drain(..).filter_map(
            |(entity, aabb, transform, mesh, material, visibility)| match meshes.get(&mesh) {
                Some(mesh) => Some((entity, aabb, transform, mesh, material, visibility)),
                _ => {
                    prepare_next_frame.push((entity, aabb, transform, mesh, material, visibility));
                    None
                }
            },
        )
    {
        let transform = transform.compute_matrix();
        let center = transform.transform_point3a(aabb.center);
        let vertices = (0..8i32)
            .map(|index| {
                let x = 2 * (index & 1) - 1;
                let y = 2 * ((index >> 1) & 1) - 1;
                let z = 2 * ((index >> 2) & 1) - 1;
                let vertex = aabb.half_extents * Vec3A::new(x as f32, y as f32, z as f32);
                transform.transform_vector3a(vertex)
            })
            .collect_vec();

        let mut min = Vec3A::ZERO;
        let mut max = Vec3A::ZERO;
        for vertex in vertices {
            min = min.min(vertex);
            max = max.max(vertex);
        }
        min += center;
        max += center;

        // Note that the `GpuInstance` is partially constructed:
        // since node index is unknown at this point.
        let min = Vec3::from(min);
        let max = Vec3::from(max);
        collection.insert(
            entity,
            (
                GpuInstance {
                    min,
                    max,
                    transform,
                    inverse_transpose_model: transform.inverse().transpose(),
                    mesh: *mesh,
                    material: 0, // TODO:
                },
                visibility,
            ),
        );
    }

    extracted_instances
        .extracted
        .append(&mut prepare_next_frame);

    if instance_changed || meshes.is_changed() {
        collection.retain(|_, (_, visibility)| Visibility::Visible == *visibility);

        let instances = collection
            .values()
            .map(|(instance, _)| instance)
            .cloned()
            .collect_vec();
        let mut instances_shapes = instances
            .iter()
            .map(|instance| GpuInstanceShape(instance.clone(), 0))
            .collect_vec();

        let instance_nodes = if collection.is_empty() {
            vec![]
        } else {
            let bvh = BVH::build(&mut instances_shapes);
            bvh.flatten_custom(&GpuNode::pack)
        };

        // TODO: I dont think this is necessary
        for ((instance, _), value) in collection.values_mut().zip_eq(instances_shapes.iter()) {
            // Assign the computed BVH node index, and mesh/material indices.
            *instance = value.0.clone();
        }

        render_assets.set(instances, instance_nodes);
        render_assets.write_buffer(&render_device, &render_queue);
    }
}

#[derive(Debug, Default, Clone, ShaderType)]
pub struct GpuInstance {
    pub min: Vec3,
    pub material: u32,
    pub max: Vec3,
    pub transform: Mat4,
    pub inverse_transpose_model: Mat4,
    pub mesh: GpuMeshIndex,
}

// Used to create BVH
struct GpuInstanceShape(GpuInstance, usize);

impl Bounded for GpuInstanceShape {
    fn aabb(&self) -> AABB {
        AABB {
            min: self.0.min.to_array().into(),
            max: self.0.max.to_array().into(),
        }
    }
}

impl BHShape for GpuInstanceShape {
    fn set_bh_node_index(&mut self, index: usize) {
        self.1 = index;
    }

    fn bh_node_index(&self) -> usize {
        self.1
    }
}

#[derive(Default, ShaderType)]
pub struct GpuInstanceBuffer {
    #[size(runtime)]
    pub data: Vec<GpuInstance>,
}
