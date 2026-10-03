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

    const DIST: f32 = 192.0;
    const TEXT_OFFSET: Vec2 = vec2(40.0, 56.0);
    const SIZE: f32 = 64.0;

    fn text(text: &'static str, dir: Vec2) -> impl Scene {
        bsn! {
            Transform
            InheritedVisibility
            Children [
                Mesh2d(asset_value(Circle::new(16.0)))
                MeshMaterial2d<ColorMaterial>(asset_value(Color::WHITE))
                Transform {
                    translation: vec3(0.0, 0.0, 1.0)
                }
                Children [
                    Mesh2d(asset_value(Circle::new(16.0)))
                    MeshMaterial2d<ColorMaterial>(asset_value(Color::BLACK))
                    Transform {
                        translation: vec3(4.0, -4.0, -0.5)
                    }
                ]
                --
                Text2d(text)
                TextFont {
                    font_size: FontSize::Px(SIZE)
                }
                Transform {
                    translation: {(TEXT_OFFSET * dir).extend(0.0)}
                }
                Text2dShadow
            ]
        }
    }

    cmd.spawn_scene_list(bsn_list! {
        Transform {
            translation: vec3(-DIST, DIST, 0.0),
        }
        @text("0,0", vec2(-1.0, 1.0))
        --
        Transform {
            translation: vec3(DIST, -DIST, 0.0),
        }
        @text("1,1", vec2(1.0, -1.0))
        --
        Transform {
            translation: vec3(-DIST, -DIST, 0.0),
        }
        @text("0,1", vec2(-1.0, -1.0))
        --
        Transform {
            translation: vec3(DIST, DIST, 0.0),
        }
        @text("1,0", vec2(1.0, 1.0))
        --
        @text("0.5,0.5", vec2(0.0, 1.25))
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
    sprite::Text2dShadow,
    sprite_render::{Material2d, Material2dPlugin},
};
