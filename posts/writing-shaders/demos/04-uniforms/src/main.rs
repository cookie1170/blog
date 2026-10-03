#[derive(AsBindGroup, Reflect, Asset, Debug, Clone)]
struct MyMaterial {
    #[uniform(0)]
    some_uniform: LinearRgba,
}

impl Material2d for MyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader.wesl".into()
    }
}

#[derive(Component, Default, Clone)]
struct Square;

fn spawn_scene(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MyMaterial>>,
) {
    cmd.spawn(Camera2d);

    let shape = meshes.add(Rectangle::new(128.0, 128.0));

    cmd.spawn((
        Square,
        Mesh2d(shape),
        MeshMaterial2d(materials.add(MyMaterial {
            some_uniform: LinearRgba::WHITE,
        })),
    ));

    cmd.spawn_scene_list(bsn_list! {
        @FeathersColorInput
        Node {
            padding: UiRect {
                left: px(8),
                top: px(8),
            },
        }
        on(color_input_self_update)
        on(|
            ev: On<ValueChange<Color>>,
            square: Single<&MeshMaterial2d<MyMaterial>, With<Square>>,
            mut assets: ResMut<Assets<MyMaterial>>
        | {
            let Some(mut mat) = assets.get_mut(&square.0) else {
                return;
            };
            mat.some_uniform = ev.value.to_linear();
        })
    })
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
            Material2dPlugin::<MyMaterial>::default(),
            FeathersPlugins,
        ))
        .insert_resource(UiTheme(feathers::dark_theme::create_dark_theme()))
        .add_systems(Startup, spawn_scene)
        .run();
}

use bevy::{
    dev_tools::{
        EasyScreenRecordPlugin, EasyScreenshotPlugin, render_debug::RenderDebugOverlayPlugin,
    },
    feathers::{
        self, FeathersPlugins,
        controls::{FeathersColorInput, color_input_self_update},
        theme::UiTheme,
    },
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
    ui_widgets::ValueChange,
};
