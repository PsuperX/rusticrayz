use super::{GpuMeshIndex, GpuMeshes, GpuNode, GpuNodeBuffer, PrepareMeshError};
use bevy::{
    prelude::*,
    render::{
        mesh::VertexAttributeValues,
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        Extract, Render, RenderApp, RenderSet,
    },
    utils::HashSet,
};
use bvh::{
    aabb::{Bounded, AABB},
    bounding_hierarchy::BHShape,
    bvh::BVH,
};
use itertools::Itertools;
use std::collections::BTreeMap;

pub struct MeshPlugin;
impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(
                    ExtractSchedule,
                    extract_mesh_assets.in_set(RenderSet::ExtractCommands),
                )
                .add_systems(Render, prepare_mesh_assets.in_set(RenderSet::PrepareAssets));
        }
    }

    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<MeshRenderAssets>();
        }
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

    let mut extracted = vec![];
    for handle in changed_assets.drain() {
        if let Some(mesh) = assets.get(&handle) {
            extracted.push((handle, mesh.clone()));
        }
    }

    commands.insert_resource(ExtractedMeshes { extracted, removed });
}

pub fn prepare_mesh_assets(
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
                    let mut vertices = [v0, v1, v2].into_iter().map(|i| GpuPrimitiveVertex {
                        position: vertices[i].position,
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

/// Container for primitive data
#[derive(Default, ShaderType)]
pub struct GpuPrimitiveBuffer {
    #[size(runtime)]
    pub data: Vec<GpuPrimitiveCompact>,
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

/// Only contains the local position of the vertex and its index in the vertex buffer
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
