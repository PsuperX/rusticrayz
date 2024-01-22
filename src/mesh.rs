use bevy::{
    prelude::*,
    render::{
        mesh::VertexAttributeValues,
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        Extract, Render, RenderApp, RenderSet,
    },
    utils::{HashMap, HashSet},
};
use bvh::{
    aabb::{Bounded, AABB},
    bounding_hierarchy::BHShape,
    bvh::BVH,
};
use itertools::Itertools;
use std::collections::BTreeMap;

use crate::instance::{GpuInstance, InstanceRenderAssets};

pub struct MeshPlugin;
impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(
                    ExtractSchedule,
                    extract_mesh_assets.in_set(RenderSet::ExtractCommands),
                )
                .add_systems(Render, prepare_mesh_assets.in_set(RenderSet::PrepareAssets))
                .add_systems(Render, queue_mesh_bind_group.in_set(RenderSet::QueueMeshes));
        }
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<GpuMeshes>()
            .init_resource::<MeshRenderAssets>()
            .init_resource::<MeshMaterialBindGroupLayout>();
    }
}

#[derive(Default, Resource)]
pub struct MeshRenderAssets {
    pub vertex_buffer: StorageBuffer<GpuVertexBuffer>,
    pub primitive_buffer: StorageBuffer<GpuPrimitiveBuffer>,
    pub node_buffer: StorageBuffer<GpuNodeBuffer>,
}

impl MeshRenderAssets {
    pub fn set(
        &mut self,
        vertices: Vec<GpuVertexCompact>,
        primitives: Vec<GpuPrimitiveCompact>,
        nodes: Vec<GpuNode>,
    ) {
        self.vertex_buffer.get_mut().data = vertices;
        self.primitive_buffer.get_mut().data = primitives;
        self.node_buffer.get_mut().count = nodes.len() as u32;
        self.node_buffer.get_mut().data = nodes;
    }

    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.vertex_buffer.write_buffer(device, queue);
        self.primitive_buffer.write_buffer(device, queue);
        self.node_buffer.write_buffer(device, queue);
    }
}

#[derive(Default, Resource)]
pub struct ExtractedMeshes {
    extracted: Vec<(Handle<Mesh>, Mesh)>,
    removed: Vec<Handle<Mesh>>,
}

fn extract_mesh_assets(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<Mesh>>>,
    assets: Extract<Res<Assets<Mesh>>>,
) {
    let mut changed_assets = HashSet::new();
    let mut removed = vec![];
    for event in events.read() {
        match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => {
                changed_assets.insert(Handle::Weak(*id));
            }
            AssetEvent::Removed { id } => {
                changed_assets.remove(&Handle::Weak(*id));
                removed.push(Handle::Weak(*id));
            }
        }
    }

    let mut extracted = Vec::new();
    for handle in changed_assets.drain() {
        if let Some(mesh) = assets.get(&handle) {
            extracted.push((handle, mesh.clone()));
        }
    }

    commands.insert_resource(ExtractedMeshes { extracted, removed });
}

fn prepare_mesh_assets(
    mut extracted_assets: ResMut<ExtractedMeshes>,
    mut assets: Local<BTreeMap<Handle<Mesh>, GpuMesh>>,
    mut meshes: ResMut<GpuMeshes>,
    mut render_assets: ResMut<MeshRenderAssets>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    if extracted_assets.removed.is_empty() && extracted_assets.extracted.is_empty() {
        return;
    }

    for handle in extracted_assets.removed.drain(..) {
        assets.remove(&handle);
        meshes.remove(&handle);
    }
    for (handle, mesh) in extracted_assets.extracted.drain(..) {
        match mesh.try_into() {
            Ok(mesh) => {
                info!("Loaded mesh {}", assets.len());
                assets.insert(handle, mesh);
            }
            Err(err) => {
                warn!("Encounter an error when loading mesh: {:#?}", err);
            }
        }
    }

    let mut vertices = vec![];
    let mut primitives = vec![];
    let mut nodes = vec![];

    for (handle, mesh) in assets.iter() {
        let vertex = vertices.len() as u32;
        let primitive = primitives.len() as u32;
        let node = UVec2::new(nodes.len() as u32, mesh.nodes.len() as u32);

        let index = GpuMeshIndex {
            vertex,
            primitive,
            node,
        };
        meshes.insert(handle.clone_weak(), index);

        vertices.extend_from_slice(&mesh.vertices);
        primitives.extend_from_slice(&mesh.primitives);
        nodes.extend_from_slice(&mesh.nodes);
    }
    render_assets.set(vertices, primitives, nodes);
    render_assets.write_buffer(&render_device, &render_queue);
}

#[derive(Resource, Deref, DerefMut)]
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

fn queue_mesh_bind_group(
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

#[derive(Default, Clone)]
pub struct GpuMesh {
    pub vertices: Vec<GpuVertexCompact>,
    pub primitives: Vec<GpuPrimitiveCompact>,
    pub nodes: Vec<GpuNode>,
}

impl TryFrom<Mesh> for GpuMesh {
    type Error = PrepareMeshError;

    fn try_from(mesh: Mesh) -> Result<Self, Self::Error> {
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .ok_or(PrepareMeshError::MissingAttributePosition)?;
        let normals = mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .and_then(VertexAttributeValues::as_float3)
            .ok_or(PrepareMeshError::MissingAttributeNormal)?;
        let uvs = mesh
            .attribute(Mesh::ATTRIBUTE_UV_0)
            .and_then(|attribute| match attribute {
                VertexAttributeValues::Float32x2(value) => Some(value),
                _ => None,
            })
            .ok_or(PrepareMeshError::MissingAttributeUV)?;

        let mut vertices = vec![];
        for (position, normal, uv) in itertools::multizip((positions, normals, uvs)) {
            vertices.push(GpuVertexCompact {
                position: Vec3::from_slice(position),
                normal: Vec3::from_slice(normal),
                u: uv[0],
                v: uv[1],
            });
        }

        let indices = match mesh.indices() {
            Some(indices) => indices.iter().collect(),
            None => vertices.iter().enumerate().map(|(id, _)| id).collect_vec(),
        };

        let primitives = match mesh.primitive_topology() {
            PrimitiveTopology::TriangleList => {
                let mut primitives = vec![];
                for chunk in &indices.iter().chunks(3) {
                    let (v0, v1, v2) = chunk
                        .cloned()
                        .next_tuple()
                        .ok_or(PrepareMeshError::IncompatiblePrimitiveTopology)?;
                    let mut vertices = [v0, v1, v2]
                        .into_iter()
                        .map(|id| vertices[id].position)
                        .zip(indices.iter())
                        .map(|(pos, &i)| GpuPrimitiveVertex {
                            position: pos,
                            index: i as u32,
                        });
                    let vertices = std::array::from_fn(|_| vertices.next().unwrap());
                    primitives.push(GpuPrimitiveCompact { vertices });
                }
                Ok(primitives)
            }
            PrimitiveTopology::TriangleStrip => {
                let primitives = indices
                    .iter()
                    .cloned()
                    .tuple_windows()
                    .enumerate()
                    .map(|(id, (v0, v1, v2))| {
                        let indices = if id & 1 == 0 {
                            [v0, v1, v2]
                        } else {
                            [v1, v0, v2]
                        };
                        let mut vertices = indices
                            .into_iter()
                            .map(|id| vertices[id].position)
                            .zip(indices.iter())
                            .map(|(pos, &i)| GpuPrimitiveVertex {
                                position: pos,
                                index: i as u32,
                            });
                        let vertices = std::array::from_fn(|_| vertices.next().unwrap());
                        GpuPrimitiveCompact { vertices }
                    })
                    .collect();
                Ok(primitives)
            }
            _ => Err(PrepareMeshError::IncompatiblePrimitiveTopology),
        }?;

        if primitives.is_empty() {
            return Err(PrepareMeshError::NoPrimitive);
        }

        let mut shapes = primitives
            .iter()
            .map(|&p| GpuPrimitiveShape(p, 0))
            .collect_vec();
        let bvh = BVH::build(&mut shapes);
        let nodes = bvh.flatten_custom(&GpuNode::pack);

        Ok(Self {
            vertices,
            primitives,
            nodes,
        })
    }
}

/// Container for vertex data
#[derive(Default, ShaderType)]
pub struct GpuVertexBuffer {
    #[size(runtime)]
    pub data: Vec<GpuVertexCompact>,
}

#[derive(Default, ShaderType)]
pub struct GpuPrimitiveBuffer {
    #[size(runtime)]
    pub data: Vec<GpuPrimitiveCompact>,
}

#[derive(Default, ShaderType)]
pub struct GpuNodeBuffer {
    pub count: u32,
    #[size(runtime)]
    pub data: Vec<GpuNode>,
}

/// A single vertex
/// This must match the Vertex definition on the shader
#[derive(Debug, Default, Clone, Copy, ShaderType)]
pub struct GpuVertexCompact {
    pub position: Vec3,
    pub u: f32,
    pub normal: Vec3,
    pub v: f32,
}

/// Only contains the GLOBAL position of the vertex and its index in the vertex buffer
/// This must match the Primitive definition on the shader
#[derive(Debug, Default, Clone, Copy, ShaderType)]
pub struct GpuPrimitiveVertex {
    pub position: Vec3,
    pub index: u32,
}

#[derive(Debug, Default, Clone, Copy, ShaderType)]
pub struct GpuPrimitiveCompact {
    pub vertices: [GpuPrimitiveVertex; 3],
}

// Used to create BVH
struct GpuPrimitiveShape(GpuPrimitiveCompact, usize);

impl Bounded for GpuPrimitiveShape {
    fn aabb(&self) -> AABB {
        AABB::empty()
            .grow(&self.0.vertices[0].position.to_array().into())
            .grow(&self.0.vertices[1].position.to_array().into())
            .grow(&self.0.vertices[2].position.to_array().into())
    }
}

impl BHShape for GpuPrimitiveShape {
    fn set_bh_node_index(&mut self, index: usize) {
        self.1 = index;
    }

    fn bh_node_index(&self) -> usize {
        self.1
    }
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

#[derive(Debug)]
pub enum PrepareMeshError {
    MissingAttributePosition,
    MissingAttributeNormal,
    MissingAttributeUV,
    IncompatiblePrimitiveTopology,
    NoPrimitive,
}
