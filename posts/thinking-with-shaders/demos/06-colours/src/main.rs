#[derive(AsBindGroup, Reflect, Asset, Debug, Clone)]
struct MyMaterial {
    #[texture(0)]
    #[sampler(1)]
    noise: Handle<Image>,
    #[uniform(2)]
    dark_colour: LinearRgba,
    #[uniform(3)]
    light_colour: LinearRgba,
}

impl Material2d for MyMaterial {
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

    let dark = Color::hsl(186.0, 0.50, 0.22);
    let light = Color::hsl(180.0, 0.51, 0.27);

    cmd.spawn((
        Mesh2d(shape),
        MeshMaterial2d(materials.add(MyMaterial {
            noise: assets.load("../../../images/noise.webp"),
            dark_colour: dark.to_linear(),
            light_colour: light.to_linear(),
        })),
    ));

    cmd.spawn_scene(bsn! {
        Node
        Children [
            @FeathersColorInput
            ColorInputValue(dark)
            on(color_input_self_update)
            on(|trigger: On<ValueChange<Color>>, mesh_material: Single<&MeshMaterial2d<MyMaterial>>, mut materials: ResMut<Assets<MyMaterial>>| -> Result<()> {
                materials.get_mut(&mesh_material.0).context("no material")?.dark_colour = trigger.value.to_linear();
                Ok(())
            })
            --
            @FeathersColorInput
            ColorInputValue(light)
            on(color_input_self_update)
            on(|trigger: On<ValueChange<Color>>, mesh_material: Single<&MeshMaterial2d<MyMaterial>>, mut materials: ResMut<Assets<MyMaterial>>| -> Result<()> {
                materials.get_mut(&mesh_material.0).context("no material")?.light_colour = trigger.value.to_linear();
                Ok(())
            })
        ]
    });
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
            FeathersPlugins,
            EasyScreenRecordPlugin::default(),
            Material2dPlugin::<MyMaterial>::default(),
        ))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(Startup, spawn_scene)
        .run();
}

use bevy::{
    asset::UnapprovedPathMode,
    dev_tools::{
        EasyScreenRecordPlugin, EasyScreenshotPlugin, render_debug::RenderDebugOverlayPlugin,
    },
    feathers::{
        FeathersPlugins,
        controls::{ColorInputValue, FeathersColorInput, color_input_self_update},
        dark_theme::create_dark_theme,
        theme::UiTheme,
    },
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
    ui_widgets::ValueChange,
};
