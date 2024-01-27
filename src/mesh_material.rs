use self::{
    instance::{GenericInstancePlugin, GpuInstance, InstancePlugin, InstanceRenderAssets},
    mesh::{GpuPrimitiveBuffer, GpuVertexBuffer, MeshPlugin, MeshRenderAssets},
};
use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice, Render, RenderApp, RenderSet},
    utils::HashMap,
};
use bvh::aabb::AABB;

mod instance;
mod mesh;

pub struct MeshMaterialPlugin;
impl Plugin for MeshMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MeshPlugin, InstancePlugin))
            .add_plugins(GenericInstancePlugin::<StandardMaterial>::default());

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<GpuMeshes>().add_systems(
                Render,
                queue_mesh_material_bind_group.in_set(RenderSet::QueueMeshes),
            );
        }
    }

    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<MeshMaterialBindGroupLayout>();
        }
    }
}

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct MeshMaterialBindGroupLayout(pub BindGroupLayout);
impl FromWorld for MeshMaterialBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("mesh_material_bindgroup_layout"),
            entries: &[
                // Vertices
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuVertexBuffer::min_size()),
                    },
                    count: None,
                },
                // Primitives
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuPrimitiveBuffer::min_size()),
                    },
                    count: None,
                },
                // Mesh nodes
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuNodeBuffer::min_size()),
                    },
                    count: None,
                },
                // Instances
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuInstance::min_size()),
                    },
                    count: None,
                },
                // Instances nodes
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuNodeBuffer::min_size()),
                    },
                    count: None,
                },
            ],
        });

        Self(layout)
    }
}

#[derive(Resource)]
pub struct MeshMaterialBindGroup {
    pub mesh_material: BindGroup,
}

fn queue_mesh_material_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    meshes: Res<MeshRenderAssets>,
    instances: Res<InstanceRenderAssets>,
    mesh_material_layout: Res<MeshMaterialBindGroupLayout>,
) {
    if let (
        Some(vertex_binding),
        Some(primitive_binding),
        Some(mesh_node_binding),
        Some(instance_binding),
        Some(instance_node_binding),
    ) = (
        meshes.vertex_buffer.binding(),
        meshes.primitive_buffer.binding(),
        meshes.node_buffer.binding(),
        instances.instance_buffer.binding(),
        instances.instance_node_buffer.binding(),
    ) {
        let mesh_material = render_device.create_bind_group(
            "mesh_material_bindgroup",
            &mesh_material_layout.0,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: vertex_binding,
                },
                BindGroupEntry {
                    binding: 1,
                    resource: primitive_binding,
                },
                BindGroupEntry {
                    binding: 2,
                    resource: mesh_node_binding,
                },
                BindGroupEntry {
                    binding: 3,
                    resource: instance_binding,
                },
                BindGroupEntry {
                    binding: 4,
                    resource: instance_node_binding,
                },
            ],
        );

        commands.insert_resource(MeshMaterialBindGroup { mesh_material });
    } else {
        commands.remove_resource::<MeshMaterialBindGroup>()
    }
}

#[derive(Debug)]
pub enum PrepareMeshError {
    MissingAttributePosition,
    MissingAttributeNormal,
    MissingAttributeUV,
    IncompatiblePrimitiveTopology,
    NoPrimitive,
}

/// Holds all GPU representatives of mesh assets.
#[derive(Default, Resource, Deref, DerefMut)]
pub struct GpuMeshes(HashMap<Handle<Mesh>, GpuMeshIndex>);

/// Offsets (and length for nodes) of the mesh in the universal buffer.
#[derive(Debug, Default, Clone, Copy, ShaderType)]
pub struct GpuMeshIndex {
    pub vertex: u32,
    pub primitive: u32,
    pub node: UVec2,
}

/// Container for nodes of a bvh
#[derive(Default, ShaderType)]
pub struct GpuNodeBuffer {
    pub count: u32,
    #[size(runtime)]
    pub data: Vec<GpuNode>,
}

/// A node in the BVH
/// This must match the Node definition on the shader
#[derive(Debug, Default, Clone, Copy, ShaderType)]
pub struct GpuNode {
    /// AABB min
    /// In case the entry_index is > 0x80000000, the AABB is undefined
    pub min: Vec3,
    /// The index of the Primitive to jump to, if the AABB test is positive.
    /// If this value is > 0x80000000 then the current node is a leaf node.
    /// Leaf nodes contain a shape index and an exit index.
    pub entry_index: u32,
    /// AABB max
    /// In case the entry_index is > 0x80000000, the AABB is undefined
    pub max: Vec3,
    /// The index of the Node to jump to, if the AABB test is negative
    pub exit_index: u32,
}

impl GpuNode {
    pub fn pack(aabb: &AABB, entry_index: u32, exit_index: u32, primitive_index: u32) -> Self {
        let entry_index = if entry_index == u32::MAX {
            primitive_index | 0x80000000
        } else {
            entry_index
        };
        let min = aabb.min.to_array().into();
        let max = aabb.max.to_array().into();
        Self {
            min,
            entry_index,
            max,
            exit_index,
        }
    }
}
