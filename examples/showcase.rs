use bevy::prelude::*;
use bevy_physimple::prelude::*;

#[derive(Default)]
struct CharacterController {
	double_jump : bool,
}

fn main() {
	let mut builder = App::build();

	builder
		.insert_resource(WindowDescriptor {
		    width: 1024.0,
		    height: 720.0,
		    title: "Physimple Showcase :)".to_string(),
		    vsync: false,
			..Default::default()
		})
		.add_plugins(DefaultPlugins)
		.add_plugin(Physics2dPlugin::default())
		.add_system(bevy::input::system::exit_on_esc_system.system())
		.add_startup_system(setup.system())
		.add_system(sensor_system.system())
		.add_system(character_system.system());
	
	builder.run();
}

fn setup(
	mut commands : Commands,
    mut materials : ResMut<Assets<ColorMaterial>>,
) {

	// Spawn a camera
	commands.spawn_bundle(OrthographicCameraBundle::new_2d());

	// Color for the floors/walls
	let black = materials.add(Color::BLACK.into());

	// Spawn a floor(s) for the player
	commands
		.spawn_bundle(SpriteBundle {
			sprite : Sprite::new(Vec2::new(500.0,20.0)),
			material : black.clone(),
			..Default::default()
		})
		.insert(StaticBody2D::new()
			.with_position(Vec2::new(0.0, -250.0))
		)
		.with_children(|p| {
			p.spawn().insert(AABB::size(Vec2::new(500.0,20.0)));
		});
	commands
		.spawn_bundle(SpriteBundle {
			sprite : Sprite::new(Vec2::new(300.0,20.0)),
			material : black.clone(),
			..Default::default()
		})
		.insert(StaticBody2D::new()
			.with_position(Vec2::new(300.0,-150.0))
		)
		.with_children(|p| {
			p.spawn().insert(AABB::size(Vec2::new(300.0,20.0)));
		});
	// Spawn a player for the floor(s)
	commands
		.spawn_bundle(SpriteBundle {
			sprite : Sprite::new(Vec2::new(20.0,24.0)),
			material : materials.add(Color::CYAN.into()),
			..Default::default()
		})
		.insert(CharacterController::default())
		.insert(KinematicBody2D::new()
			.with_position(Vec2::new(0.0, 0.0))
			.with_terminal(Vec2::new(250.0,1000.0))
		)
		.with_children(|p| {
			p.spawn().insert(AABB::size(Vec2::new(20.0,24.0)));
		});
	
	// Spawn a wall for the player to jump on :D
	commands
		.spawn_bundle(SpriteBundle {
			sprite : Sprite::new(Vec2::new(20.0, 300.0)),
			material : black.clone(),
			..Default::default()
		})
		.insert(
			StaticBody2D::new()
				.with_position(Vec2::new(440.0,0.0))
		)
		.with_children(|p| {
			p.spawn().insert(AABB::size(Vec2::new(20.0,300.0)));
		});

	// Spawn a bouncy floor at the other side
	commands
		.spawn_bundle(SpriteBundle {
			sprite : Sprite::new(Vec2::new(60.0,20.0)),
			material : materials.add(Color::PURPLE.into()),
			..Default::default()
		})
		.insert(
			StaticBody2D::new()
				.with_position(Vec2::new(-400.0,-100.0))
				.with_bounciness(1.0)
		)
		.with_children(|p| {
			p.spawn().insert(AABB::size(Vec2::new(60.0,20.0)));
		});
	// Spawn a cube to bounce on
	commands
		.spawn_bundle(SpriteBundle {
			sprite : Sprite::new(Vec2::new(20.0,20.0)),
			material : materials.add(Color::MIDNIGHT_BLUE.into()),
			..Default::default()
		})
		.insert(
			KinematicBody2D::new()
				.with_stiffness(1.0)
				.with_position(Vec2::new(-400.0, 100.0))
		)
		.with_children(|p| {
			p.spawn().insert(AABB::size(Vec2::new(20.0,20.0)));
		});
	// Spawn a sensor to change color based on the sprite bundle

	commands
		.spawn_bundle(SpriteBundle {
			sprite : Sprite::new(Vec2::new(50.0,50.0)),
			material : materials.add(Color::rgba(1.0,0.0,0.0,0.5).into()),
			..Default::default()
		})
		.insert(
			Sensor2D::new()
				.with_position(Vec2::new(-400.0,80.0))
		)
		.with_children(|p| {
			p.spawn().insert(AABB::size(Vec2::new(50.0,50.0)));
		});


	// spawn a tower of cubes
	let color_1 = materials.add(Color::PINK.into());
	let color_2 = materials.add(Color::GOLD.into());

	(0..10).for_each(|i| {
		let color = if i % 2 == 0 { color_1.clone() } else { color_2.clone() };

		commands
			.spawn_bundle(SpriteBundle {
				sprite : Sprite::new(Vec2::new(20.0,20.0)),
				material : color,
				..Default::default()
			})
			.insert(
				KinematicBody2D::new()
					.with_position(Vec2::new(350.0 + i as f32, i as f32 * 60.0))
					.with_stiffness(0.9)
			)
			.with_children(|p| {
				p.spawn().insert(AABB::size(Vec2::new(20.0,20.0)));
			});
	});
}

fn character_system(
    input: Res<Input<KeyCode>>,
    phys_sets : Res<PhysicsSettings>,
    mut query: Query<(&mut CharacterController, &mut KinematicBody2D)>,
) {
    let gravity = phys_sets.gravity;

    for (mut controller, mut body) in query.iter_mut() {
        if let Some(normal) = body.on_wall() {
			// if we are sliding across a wall, feed -0.1 * wall normal
			// to stick to the wall
            body.linvel -= normal * 0.1;

			const WALL_SLIDE_SPEED : f32 = -25.0;
            if body.linvel.y < WALL_SLIDE_SPEED {
                body.linvel.y = WALL_SLIDE_SPEED;
            }
        }

		// This is just a weird way to do jump, using the gravity direction and size(tho you dont need the size)
		// it works by sliding on the gravity direction(so nothing in the direction of gravity)
		// then adding the jump force(here its gravity * 0.5) to the velocity
        let jump = |body : &mut KinematicBody2D| {
            body.linvel = body.linvel.slide(gravity.normalize()) - gravity * 0.6;
            let wall = body.on_wall().unwrap_or(Vec2::ZERO) * 250.0;
            body.linvel += wall;
        };

        let should_jump = input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::W);
        if body.on_floor().is_some() || body.on_wall().is_some() {
            controller.double_jump = true;

            if should_jump {
                jump(&mut body);
            }
        }
        else if controller.double_jump && should_jump {
            controller.double_jump = false;
            jump(&mut body);
        }
        // It might look like we need to multiply by delta_time but the apply_force function does it for us(in the physics step)
        let acc = Vec2::new(1000.0,0.0);
        if input.pressed(KeyCode::A) {
            body.apply_force(-acc);
            // body.apply_angular_impulse(1.0);
        }
        if input.pressed(KeyCode::D) {
            body.apply_force(acc);
            // body.apply_angular_impulse(-1.0);
        }
    }
}

fn sensor_system(
	mut sensors : Query<(&Sensor2D, &mut Handle<ColorMaterial>)>,
	mut materials : ResMut<Assets<ColorMaterial>>,
) {
	for (sensor, color) in sensors.iter_mut() {
		if sensor.iter_overlapping_bodies().count() == 0 {
			let _ = materials.set(color.id, Color::rgba(1.0,0.0,0.0,0.5).into());
		}
		else {
			let _ = materials.set(color.id, Color::rgba(0.0,1.0,0.0,0.5).into());
		}
	}
}
