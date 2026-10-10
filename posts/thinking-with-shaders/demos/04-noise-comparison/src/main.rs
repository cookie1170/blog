#[derive(AsBindGroup, Reflect, Asset, Debug, Clone)]
struct MyMaterial {}

impl Material2d for MyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader.wesl".into()
    }
}

fn spawn_scene(
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MyMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut cmd: Commands,
) {
    cmd.spawn(Camera2d);

    let shape = meshes.add(Rectangle::new(384.0, 384.0));

    cmd.spawn((
        Transform::from_xyz(-256.0, 0.0, 0.0),
        Mesh2d(shape.clone()),
        MeshMaterial2d(materials.add(MyMaterial {})),
    ));

    cmd.spawn((
        Transform::from_xyz(256.0, 0.0, 0.0),
        Mesh2d(shape),
        MeshMaterial2d(color_materials.add(ColorMaterial {
            texture: Some(assets.load("../../../images/noise.webp")),
            ..default()
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
    sprite_render::{Material2d, Material2dPlugin},
};
