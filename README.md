# Rusticrayz

A high-performance path tracing engine in Rust using WGSL compute shaders for GPU-accelerated rendering. It replaces Bevy’s default pipeline and integrates a Bounding Volume Hierarchy (BVH) structure to optimize ray tracing.
Designed for real-time, photorealistic rendering, the engine handles complex lighting and materials with parallel GPU
computation.

## Features

- Custom raytracer implementation
- Fly camera for easy navigation
- Ability to switch between raytracer and default Bevy 3D rendering
- World inspector for debugging and scene exploration
- Texture and material support

## Getting Started

1. Clone the repository:
```bash
git clone https://github.com/PsuperX/rusticrayz.git
cd rusticrayz
```

2. Build and run the project:
```bash
cargo run --release
```

3. Explore different scenes:
```bash
cargo run --release --example cornell_box
```

## Usage

- Use WASD keys and mouse to navigate the 3D environment (fly camera).
- Press 'C' to switch between the custom raytracer and Bevy's default 3D rendering.
- Press 'R' to reset the camera position.
- Use the world inspector (provided by bevy_inspector_egui) for debugging and exploring the scene.

## Acknowledgements

- [Bevy](https://bevyengine.org/) - The game engine powering this project
- [bevy_flycam](https://github.com/sburris0/bevy_flycam) - For camera controls
- [bevy_inspector_egui](https://github.com/jakobhellermann/bevy-inspector-egui) - For the world inspector functionality