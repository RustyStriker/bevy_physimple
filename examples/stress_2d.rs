use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_physimple::prelude2d::*;

fn main() {
    let mut builder = App::build();
    builder
		.insert_resource(WindowDescriptor {
		    width: 500.0,
		    height: 500.0,
		    title: "!!! Stress test !!!".to_string(),
		    vsync: false,
		    ..Default::default()
		})
        .add_plugins(DefaultPlugins)
        .add_plugin(Physics2dPlugin)
        .insert_resource(GlobalGravity(Vec2::new(0.0, -100.0)))
        .insert_resource(GlobalFriction(0.90))
        .insert_resource(GlobalStep(0.0))
        .insert_resource(GlobalUp(Vec2::new(0.0, 1.0)))
        .add_startup_system(setup.system())
		.add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system(bevy::input::system::exit_on_esc_system.system());
    // builder.add_system(character_system.system());
    builder.run();
}

fn setup(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
	let black = materials.add(Color::BLACK.into());

	// spawn a camera
	commands.spawn_bundle(OrthographicCameraBundle::new_2d());
	
	// Spawn a couple of walls
	commands
		.spawn_bundle(SpriteBundle {
			sprite : Sprite::new(Vec2::new(500.0,20.0)),
			material : black.clone(),
			..Default::default()
		})
		.insert(StaticBody2D::new()
				.with_position(Vec2::new(0.0,250.0))
		)
		.with_children(|p| {
			p.spawn().insert(AABB::size(Vec2::new(500.0,20.0)));
		});
	commands
		.spawn_bundle(SpriteBundle {
			sprite : Sprite::new(Vec2::new(500.0,20.0)),
			material : black.clone(),
			..Default::default()
		})
		.insert(StaticBody2D::new()
				.with_position(Vec2::new(0.0,-250.0))
		)
		.with_children(|p| {
			p.spawn().insert(AABB::size(Vec2::new(500.0,20.0)));
		});
	commands
		.spawn_bundle(SpriteBundle {
			sprite : Sprite::new(Vec2::new(20.0,500.0)),
			material : black.clone(),
			..Default::default()
		})
		.insert(StaticBody2D::new()
				.with_position(Vec2::new(250.0,0.0))
		)
		.with_children(|p| {
			p.spawn().insert(AABB::size(Vec2::new(20.0,500.0)));
		});
	commands
		.spawn_bundle(SpriteBundle {
			sprite : Sprite::new(Vec2::new(20.0,500.0)),
			material : black.clone(),
			..Default::default()
		})
		.insert(StaticBody2D::new()
				.with_position(Vec2::new(-250.0,0.0))
		)
		.with_children(|p| {
			p.spawn().insert(AABB::size(Vec2::new(20.0,500.0)));
		});


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

			commands
				.spawn_bundle(SpriteBundle {
					sprite : Sprite::new(Vec2::new(SCALE,SCALE)),
					material : color,
					..Default::default()
				})
				.insert(KinematicBody2D::new()
							.with_position(Vec2::new(4.0 - i as f32, 4.0 - k as f32) * SCALE * 2.0)
				)
				.with_children(|p| {
					p.spawn().insert(AABB::size(Vec2::new(SCALE,SCALE)));
				});
		})
	})
}
