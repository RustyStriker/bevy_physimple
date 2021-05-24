use bevy::prelude::*;
use bevy_physimple::prelude::*;

#[derive(Default)]
pub struct CharacterController {
    double_jump : bool
}

fn main() {
    let mut builder = App::build();
    builder
        .add_plugins(DefaultPlugins)
        .add_plugin(Physics2dPlugin::default())
        .add_startup_system(setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system());
    builder.add_system(character_system.system());
    builder.run();
}

fn setup(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // let icon = asset_server.load("icon.png");
    // let plat = asset_server.load("platform.png");
    // let square = asset_server.load("square.png");

    let blue = materials.add(Color::ALICE_BLUE.into());
    let black = materials.add(Color::BLACK.into());
    let another_color = materials.add(Color::GOLD.into());

    // Spawn the damn camera
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d());


    // Spawn character
    // let _player_id = commands
    //     .spawn_bundle(SpriteBundle {
    //         sprite : Sprite::new(Vec2::new(28.0,28.0)),
    //         material: blue.clone(),
    //         ..Default::default()
    //     })
    //     .insert(
    //         KinematicBody2D::new()
    //             .with_terminal(Vec2::new(400.0, 1000.0))
    //             .with_mask(3)
    //             .with_friction(1.5)
    //     )
    //     .insert(CharacterController::default())
    //     .insert(Square::size(Vec2::new(28.0,28.0)))
	// 	.id();
    
    // center floor
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(600.0,20.0)),
            material: black.clone(),
            transform : Transform::from_xyz(150.0,-200.0,0.0),
            ..Default::default()
        })
        .insert(
            StaticBody2D::new()
                .with_layer(3)
        )
		.insert(Square::size(Vec2::new(600.0,20.0)));

    // Spawn the cube near us
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::splat(20.0)),
            material: another_color.clone(),
            transform : Transform::from_xyz(30.0,60.0,0.0),
            ..Default::default()
        })
        .insert(
            KinematicBody2D::new()
                .with_mass(2.0)
                .with_friction(0.1) // Basically almost no friction, should be fun :D
                .with_bounciness(0.9) // Make it bouncy(also on walls)
                .with_linear_velocity(Vec2::new(220.0,0.0))
		)
		.insert(Square::size(Vec2::new(20.0, 20.0)));

    // Circles

    // commands
    //     .spawn_bundle(SpriteBundle {
    //         sprite : Sprite::new(Vec2::splat(40.0)),
    //         material : another_color.clone(),
    //         transform : Transform::from_xyz(-100.0, 0.0, 0.0),
    //         ..Default::default()
    //     })
    //     .insert(
    //         KinematicBody2D::new()
    //     )
    //     .insert(Circle::new(20.0));

    // commands.spawn_bundle(SpriteBundle {
    //     sprite : Sprite::new(Vec2::splat(40.0)),
    //     material : black.clone(),
    //     transform : Transform::from_xyz(-100.0, -200.0,0.0),
    //     ..Default::default()
    // })
    // .insert(StaticBody2D::new())
    // .insert(Circle::new(20.0));
}

fn character_system(
    input: Res<Input<KeyCode>>,
    phys_sets : Res<PhysicsSettings>,
    mut query: Query<(&mut CharacterController, &mut KinematicBody2D)>,
) {
    let gravity = phys_sets.gravity;

    for (mut controller, mut body) in query.iter_mut() {
        if let Some(normal) = body.on_wall() {
            body.linvel -= normal * 0.1;

            if body.linvel.y < -1.0 {
                body.linvel.y = -1.0;
            }
        }

        let jump = |body : &mut KinematicBody2D| {
            body.linvel = body.linvel.slide(gravity.normalize()) - gravity * 0.6;
            let wall = body.on_wall().unwrap_or(Vec2::ZERO) * 250.0;
            body.linvel += wall;
        };

        let should_jump = input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::W);
        if body.on_floor().is_some() || body.on_wall().is_some() {
            controller.double_jump = true;

            if should_jump {
                // This is just a weird way to do jump, using the gravity direction and size(tho you dont need the size)
                // it works by sliding on the gravity direction(so nothing in the direction of gravity)
                // then adding the jump force(here its gravity * 0.5) to the velocity
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
