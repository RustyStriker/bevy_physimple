use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_physimple::prelude::*;

struct StartInTime {
    seconds : f64,
}

fn main() {
    println!("=========================================================");
    println!("     Currently not working due to bad Collision code     ");
    println!("=========================================================");

    let mut builder = App::build();
    builder
        .insert_resource(WindowDescriptor {
            width : 500.0,
            height : 500.0,
            title : "!!! Stress test !!!".to_string(),
            vsync : false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Physics2dPlugin::default())
        .add_startup_system(setup.system())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system(enable_system.system())
        .add_system(bevy::input::system::exit_on_esc_system.system());

    builder.run();
}

fn setup(
    mut commands : Commands,
    // asset_server: Res<AssetServer>,
    mut materials : ResMut<Assets<ColorMaterial>>,
) {
    let black = materials.add(Color::BLACK.into());
    let purple = materials.add(Color::PURPLE.into());

    // spawn a camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Spawn a couple of walls
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(500.0, 20.0)),
            material : black.clone(),
            transform : Transform::from_xyz(0.0, 250.0, 0.0),
            ..Default::default()
        })
        .insert(StaticBody2D::new())
        .insert(Square::size(Vec2::new(500.0, 20.0)));
    commands // FLOOR
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(500.0, 20.0)),
            material : purple.clone(),
            transform : Transform::from_xyz(0.0, -250.0, 0.0),
            ..Default::default()
        })
        .insert(StaticBody2D::new().with_bounciness(0.9))
        .insert(Square::size(Vec2::new(500.0, 20.0)));
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(20.0, 500.0)),
            material : black.clone(),
            transform : Transform::from_xyz(250.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(StaticBody2D::new())
        .insert(Square::size(Vec2::new(20.0, 500.0)));
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(20.0, 500.0)),
            material : black.clone(),
            transform : Transform::from_xyz(-250.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(StaticBody2D::new())
        .insert(Square::size(Vec2::new(20.0, 500.0)));

    let cyan = materials.add(Color::CYAN.into());
    let green = materials.add(Color::LIME_GREEN.into());

    const SCALE : f32 = 20.0;
    // spawn around 36 cubes
    (0..8).for_each(|i| {
        (0..8).for_each(|k| {
            let color = if (i + k) % 2 == 0 {
                cyan.clone()
            }
            else {
                green.clone()
            };

            let pos = Vec3::new(4.0 - i as f32, 4.0 - k as f32, 0.0) * SCALE * 2.0;

            commands
                .spawn_bundle(SpriteBundle {
                    sprite : Sprite::new(Vec2::splat(SCALE)),
                    material : color,
                    transform : Transform::from_xyz(pos.x, pos.y, pos.z),
                    ..Default::default()
                })
                .insert(
                    KinematicBody2D::new()
                        .with_active(false)
                        .with_stiffness(0.9),
                )
                .insert(StartInTime {
                    seconds : (8 - k) as f64,
                })
                .insert(Square::size(Vec2::splat(SCALE)));
        })
    })
}

fn enable_system(
    time : Res<Time>,
    mut query : Query<(&mut KinematicBody2D, &StartInTime)>,
) {
    let time = time.seconds_since_startup();
    query
        .iter_mut()
        .filter(|(b, t)| !b.active && t.seconds < time)
        .for_each(|(mut b, _)| b.active = true);
}
