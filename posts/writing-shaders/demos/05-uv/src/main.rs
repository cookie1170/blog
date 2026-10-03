#[derive(AsBindGroup, Reflect, Asset, Debug, Clone)]
struct MyMaterial {
    #[texture(0)]
    #[sampler(1)]
    image: Handle<Image>,
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

    cmd.spawn((
        Mesh2d(shape),
        MeshMaterial2d(materials.add(MyMaterial {
            image: assets.load("grass.png"),
        })),
    ));

    const HORIZONTAL: f32 = 192.0;
    const VERTICAL: f32 = 232.0;
    const SIZE: f32 = 64.0;

    cmd.spawn_scene_list(bsn_list! {
        Text2d("0,0")
        TextFont {
            font_size: FontSize::Px(SIZE)
        }
        Transform {
            translation: vec3(-HORIZONTAL, VERTICAL, 0.0),
        }
        --
        Text2d("1,1")
        Transform {
            translation: vec3(HORIZONTAL, -VERTICAL, 0.0),
        }
        TextFont {
            font_size: FontSize::Px(SIZE)
        }
        --
        Text2d("0,1")
        Transform {
            translation: vec3(-HORIZONTAL, -VERTICAL, 0.0),
        }
        TextFont {
            font_size: FontSize::Px(SIZE)
        }
        TextFont {
            font_size: FontSize::Px(SIZE)
        }        --
        Text2d("1,0")
        Transform {
            translation: vec3(HORIZONTAL, VERTICAL, 0.0),
        }
        TextFont {
            font_size: FontSize::Px(SIZE)
        }
        --
        Text2d("0.5,0.5")
        TextFont {
            font_size: FontSize::Px(SIZE)
        }
    });
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
        ))
        .add_systems(Startup, spawn_scene)
        .run();
}

use bevy::{
    dev_tools::{
        EasyScreenRecordPlugin, EasyScreenshotPlugin, render_debug::RenderDebugOverlayPlugin,
    },
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};
