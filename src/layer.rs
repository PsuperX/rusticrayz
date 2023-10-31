use crate::{app::WgpuCtx, scene::Scene};

pub trait Layer {
    fn on_attach(&mut self, _ctx: &mut WgpuCtx) {}
    fn on_detach(&mut self) {}

    fn on_ui_render(&mut self, _ctx: &egui::Context) {}

    fn on_draw_frame(
        &mut self,
        ctx: &WgpuCtx,
        view: &wgpu::TextureView,
        scene: &Scene,
    ) -> wgpu::CommandBuffer;

    fn on_resize(&mut self, ctx: &mut WgpuCtx);
}
