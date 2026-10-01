fn spawn_scene(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    cmd.spawn(Camera2d);

    let shape = meshes.add(Rectangle::new(128.0, 128.0));

    cmd.spawn((Mesh2d(shape), MeshMaterial2d(materials.add(Color::WHITE))));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().disable::<RenderDebugOverlayPlugin>(),
            EasyScreenshotPlugin {
                trigger: KeyCode::F12,
                ..default()
            },
            EasyScreenRecordPlugin::default(),
        ))
        .add_systems(Startup, spawn_scene)
        .run();
}

use bevy::{
    dev_tools::{
        EasyScreenRecordPlugin, EasyScreenshotPlugin, render_debug::RenderDebugOverlayPlugin,
    },
    prelude::*,
};
