use bevy::{
    input::common_conditions::input_toggle_active,
    prelude::*,
    render::{
        camera::{RenderTarget, ScalingMode},
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::BevyDefault,
        view::RenderLayers,
    },
    sprite::MaterialMesh2dBundle,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

pub const WIDTH: f32 = 857.0;
pub const HEIGHT: f32 = 480.0;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Pizza Survivor".into(),
                        resolution: (WIDTH, HEIGHT).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_plugin(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_camera)
        .add_system(player_movement)
        .run();
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub health: f32,
}

#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
    pub health: f32,
}

#[derive(Resource)]
pub struct MainRender(pub Handle<Image>);

#[derive(Component)]
pub struct MainCamera;

fn spawn_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 1920,
        height: 1080,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);

    let image_handle = images.add(image);

    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedVertical(20.0);
    camera.camera.target = RenderTarget::Image(image_handle.clone());

    commands.spawn((camera, MainCamera, UiCameraConfig { show_ui: true }));

    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(16.0, 9.0))));

    let material_handle = materials.add(ColorMaterial {
        texture: Some(image_handle.clone()),
        ..default()
    });

    let post_processing_pass_layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: quad_handle.into(),
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..default()
            },
            ..default()
        },
        post_processing_pass_layer,
        Name::new("Base Render"),
    ));

    commands.insert_resource(MainRender(image_handle));

    let mut camera = Camera2dBundle::default();
    camera.camera.order = 999;
    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: 16.0,
        min_height: 9.0,
    };

    commands.spawn((
        camera,
        post_processing_pass_layer,
        UiCameraConfig { show_ui: false },
    ));
}

pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        SpriteBundle::default(),
        Player {
            speed: 10.0,
            health: 100.0,
        },
        Collider::ball(1.0),
    ));

    for i in 0..10 {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    ..default()
                },
                transform: Transform::from_xyz(1.0 + 0.1 * i as f32, 1.0, 100.0),
                ..default()
            },
            Enemy {
                speed: 5.0,
                health: 5.0,
            },
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED_Z,
            Damping {
                linear_damping: 100.0,
                angular_damping: 1.0,
            },
            Collider::ball(1.0),
        ));
    }
}

fn player_movement(
    mut player: Query<(&mut Transform, &Player)>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut transform, player) = player.single_mut();
    if input.pressed(KeyCode::W) {
        transform.translation.y += time.delta_seconds() * player.speed;
    }
    if input.pressed(KeyCode::S) {
        transform.translation.y -= time.delta_seconds() * player.speed;
    }
    if input.pressed(KeyCode::A) {
        transform.translation.x -= time.delta_seconds() * player.speed;
    }
    if input.pressed(KeyCode::D) {
        transform.translation.x += time.delta_seconds() * player.speed;
    }
}
