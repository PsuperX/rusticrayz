use self::{
    instance::{GenericInstancePlugin, GpuInstance, InstancePlugin, InstanceRenderAssets},
    material::{GenericMaterialPlugin, GpuStandardMaterial, MaterialPlugin, MaterialRenderAssets},
    mesh::{GpuPrimitiveBuffer, GpuVertexBuffer, MeshPlugin, MeshRenderAssets},
};
use bevy::{
    pbr::MeshPipeline,
    prelude::*,
    render::{
        render_asset::RenderAssets, render_resource::*, renderer::RenderDevice, Render, RenderApp,
        RenderSet,
    },
    utils::HashMap,
};
use bvh::aabb::AABB;
use itertools::Itertools;
use std::{iter, num::NonZeroU32};

mod instance;
mod material;
mod mesh;

pub struct MeshMaterialPlugin;
impl Plugin for MeshMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MeshPlugin, MaterialPlugin, InstancePlugin))
            .add_plugins(GenericMaterialPlugin::<StandardMaterial>::default())
            .add_plugins(GenericInstancePlugin::<StandardMaterial>::default());

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<GpuMeshes>()
                .add_systems(
                    Render,
                    prepare_texture_bind_group_layout
                        .in_set(RenderSet::Queue)
                        .before(queue_mesh_material_bind_group),
                )
                .add_systems(
                    Render,
                    queue_mesh_material_bind_group.in_set(RenderSet::QueueMeshes),
                );
        }
    }

    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<MeshMaterialBindGroupLayout>()
                .init_resource::<TextureBindGroupLayout>();
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
                // Materials
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuStandardMaterial::min_size()),
                    },
                    count: None,
                },
                // Instances
                BindGroupLayoutEntry {
                    binding: 4,
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
                    binding: 5,
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

#[derive(Resource, Clone)]
pub struct TextureBindGroupLayout {
    pub layout: BindGroupLayout,
    pub texture_count: u32,
}
impl TextureBindGroupLayout {
    fn get_layout(render_device: &RenderDevice, texture_count: NonZeroU32) -> Self {
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("mesh_material_bindgroup_layout"),
            entries: &[
                // Textures
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: Some(texture_count),
                },
                // Samplers
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: Some(texture_count),
                },
            ],
        });

        Self {
            layout,
            texture_count: texture_count.into(),
        }
    }
}

impl FromWorld for TextureBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        Self::get_layout(render_device, NonZeroU32::new(1).unwrap())
    }
}

fn prepare_texture_bind_group_layout(
    render_device: Res<RenderDevice>,
    materials: Res<MaterialRenderAssets>,
    mut texture_layout: ResMut<TextureBindGroupLayout>,
) {
    *texture_layout = TextureBindGroupLayout::get_layout(
        &render_device,
        NonZeroU32::new(materials.textures.len() as u32 + 1).unwrap(),
    );
}

#[derive(Resource)]
pub struct MeshMaterialBindGroup {
    pub mesh_material: BindGroup,
    pub textures: BindGroup,
}

#[allow(clippy::too_many_arguments)]
fn queue_mesh_material_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mesh_pipeline: Res<MeshPipeline>,
    meshes: Res<MeshRenderAssets>,
    materials: Res<MaterialRenderAssets>,
    instances: Res<InstanceRenderAssets>,
    images: Res<RenderAssets<Image>>,
    mesh_material_layout: Res<MeshMaterialBindGroupLayout>,
    texture_layout: Res<TextureBindGroupLayout>,
) {
    if let (
        Some(vertex_binding),
        Some(primitive_binding),
        Some(mesh_node_binding),
        Some(material_binding),
        Some(instance_binding),
        Some(instance_node_binding),
    ) = (
        meshes.vertex_buffer.binding(),
        meshes.primitive_buffer.binding(),
        meshes.node_buffer.binding(),
        materials.materials.binding(),
        instances.instance_buffer.binding(),
        instances.instance_node_buffer.binding(),
    ) {
        let mesh_material = render_device.create_bind_group(
            "mesh_material_bindgroup",
            &mesh_material_layout.0,
            &BindGroupEntries::sequential((
                vertex_binding,
                primitive_binding,
                mesh_node_binding,
                material_binding,
                instance_binding,
                instance_node_binding,
            )),
        );

        let images = materials
            .textures
            .iter()
            .map(|handle| {
                images
                    .get(handle)
                    .unwrap_or(&mesh_pipeline.dummy_white_gpu_image)
            })
            .chain(iter::once(&mesh_pipeline.dummy_white_gpu_image)); // TODO: find a better solution
        let textures = images
            .clone()
            .map(|image| &*image.texture_view)
            .collect_vec();
        let samplers = images.map(|image| &*image.sampler).collect_vec();

        let textures = render_device.create_bind_group(
            "texture_bindgroup",
            &texture_layout.layout,
            &BindGroupEntries::sequential((
                BindingResource::TextureViewArray(&textures),
                BindingResource::SamplerArray(&samplers),
            )),
        );

        commands.insert_resource(MeshMaterialBindGroup {
            mesh_material,
            textures,
        });
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

/// Holds the indices of the GPU representatives of mesh assets.
#[derive(Default, Resource, Deref, DerefMut)]
pub struct GpuMeshes(HashMap<Handle<Mesh>, GpuMeshIndex>);

/// Offsets (and length for nodes) of the mesh in the universal buffer.
#[derive(Debug, Default, Clone, Copy, ShaderType)]
pub struct GpuMeshIndex {
    pub vertex: u32,
    pub primitive: u32,
    pub node: UVec2,
}

/// Holds the indices of the GPU representatives of material assets.
#[derive(Default, Resource, Deref, DerefMut)]
pub struct GpuStandardMaterials(HashMap<UntypedHandle, u32>);

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
