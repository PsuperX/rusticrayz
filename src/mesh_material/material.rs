use super::GpuStandardMaterials;
use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        Extract, Render, RenderApp, RenderSet,
    },
    utils::{HashMap, HashSet},
};
use indexmap::set::IndexSet;
use std::marker::PhantomData;

pub struct MaterialPlugin;
impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<GpuStandardMaterials>()
                .init_resource::<MaterialRenderAssets>()
                .add_systems(
                    Render,
                    prepare_material_assets.in_set(RenderSet::PrepareAssets),
                );
        }
    }
}

#[derive(Default)]
pub struct GenericMaterialPlugin<M: Into<StandardMaterial>>(PhantomData<M>);
impl<M> Plugin for GenericMaterialPlugin<M>
where
    M: Into<StandardMaterial> + Asset + Clone,
{
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(
                ExtractSchedule,
                extract_materials_assets::<M>.in_set(RenderSet::ExtractCommands),
            );
        }
    }
}

#[derive(Default, Resource)]
pub struct MaterialRenderAssets {
    pub materials: StorageBuffer<GpuStandardMaterialBuffer>,
    pub textures: Vec<Handle<Image>>,
}

#[derive(Default, Resource)]
pub struct ExtractedMaterials {
    extracted: Vec<(UntypedHandle, StandardMaterial)>,
    removed: Vec<UntypedHandle>,
}

fn extract_materials_assets<M: Into<StandardMaterial> + Asset + Clone>(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<M>>>,
    assets: Extract<Res<Assets<M>>>,
) {
    let mut changed_assets = HashSet::new();
    let mut removed = vec![];
    for event in events.read() {
        match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => {
                changed_assets.insert(UntypedHandle::Weak(id.untyped()));
            }
            AssetEvent::Removed { id } => {
                changed_assets.remove(&UntypedHandle::Weak(id.untyped()));
                removed.push(UntypedHandle::Weak(id.untyped()));
            }
        }
    }

    let mut extracted = vec![];
    for handle in changed_assets.drain() {
        if let Some(material) = assets.get(&handle) {
            extracted.push((handle, material.clone().into()));
        }
    }

    commands.insert_resource(ExtractedMaterials { extracted, removed });
}

pub fn prepare_material_assets(
    mut extracted_assets: ResMut<ExtractedMaterials>,
    mut assets: Local<HashMap<UntypedHandle, StandardMaterial>>,
    mut materials: ResMut<GpuStandardMaterials>,
    mut render_assets: ResMut<MaterialRenderAssets>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    if extracted_assets.removed.is_empty() && extracted_assets.extracted.is_empty() {
        return;
    }

    for handle in extracted_assets.removed.drain(..) {
        assets.remove(&handle);
        materials.remove(&handle);
    }
    for (handle, material) in extracted_assets.extracted.drain(..) {
        assets.insert(handle, material);
    }

    let mut textures = IndexSet::new();
    let materials = assets
        .iter()
        .enumerate()
        .map(|(index, (handle, material))| {
            add_textures(&mut textures, material);

            let get_index = |maybe_handle: &Option<Handle<Image>>| {
                maybe_handle
                    .as_ref()
                    .and_then(|handle| textures.get_index_of(handle).map(|i| i as u32))
                    .unwrap_or(u32::MAX)
            };

            let material = GpuStandardMaterial {
                base_color: material.base_color.into(),
                base_color_texture: get_index(&material.base_color_texture),
                emissive: material.emissive.into(),
                emissive_texture: get_index(&material.emissive_texture),
                perceptual_roughness: material.perceptual_roughness,
                metallic: material.metallic,
                metallic_roughness_texture: get_index(&material.metallic_roughness_texture),
                reflectance: material.reflectance,
                normal_map_texture: get_index(&material.normal_map_texture),
            };
            materials.insert(handle.clone_weak(), index as u32);
            material
        })
        .collect();

    render_assets.textures.clear();
    render_assets.textures.extend(textures);
    render_assets.materials.get_mut().data = materials;
    render_assets
        .materials
        .write_buffer(&render_device, &render_queue);
}

fn add_textures(textures: &mut IndexSet<Handle<Image>>, material: &StandardMaterial) {
    let to_add = [
        &material.base_color_texture,
        &material.emissive_texture,
        &material.metallic_roughness_texture,
        &material.normal_map_texture,
    ];
    for texture in to_add.into_iter().flatten() {
        textures.insert(texture.clone_weak());
    }
}

#[derive(Debug, ShaderType)]
pub struct GpuStandardMaterial {
    pub base_color: Vec4,
    pub base_color_texture: u32,
    pub emissive: Vec4,
    pub emissive_texture: u32,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub metallic_roughness_texture: u32,
    pub reflectance: f32,
    pub normal_map_texture: u32,
}

/// Container for vertex data
#[derive(Default, ShaderType)]
pub struct GpuStandardMaterialBuffer {
    #[size(runtime)]
    pub data: Vec<GpuStandardMaterial>,
}
