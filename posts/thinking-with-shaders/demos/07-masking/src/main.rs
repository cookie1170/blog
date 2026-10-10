#[derive(AsBindGroup, Reflect, Asset, Debug, Clone)]
struct MyMaterial {
    #[texture(0)]
    #[sampler(1)]
    noise: Handle<Image>,
    #[uniform(2)]
    colour: LinearRgba,
}

impl Material2d for MyMaterial {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn fragment_shader() -> ShaderRef {
        "shader.wesl".into()
    }
}

fn spawn_scene(
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MyMaterial>>,
    mut cmd: Commands,
) {
    cmd.spawn(Camera2d);

    let shape = meshes.add(Rectangle::new(384.0, 384.0));

    let colour = Color::hsl(180.0, 0.51, 0.27);

    cmd.spawn((
        Mesh2d(shape),
        MeshMaterial2d(materials.add(MyMaterial {
            noise: assets.load("../../../images/noise.webp"),
            colour: colour.to_linear(),
        })),
    ));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .disable::<RenderDebugOverlayPlugin>()
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                }),
            EasyScreenshotPlugin {
                trigger: KeyCode::F12,
                ..default()
            },
            EasyScreenRecordPlugin::default(),
            Material2dPlugin::<MyMaterial>::default(),
        ))
        .add_systems(Startup, spawn_scene)
        .run();
}

use bevy::{
    asset::UnapprovedPathMode,
    dev_tools::{
        EasyScreenRecordPlugin, EasyScreenshotPlugin, render_debug::RenderDebugOverlayPlugin,
    },
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};
