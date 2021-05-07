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
    let _player_id = commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(28.0,28.0)),
            material: blue.clone(),
            ..Default::default()
        })
        .insert(
            KinematicBody2D::new()
                .with_terminal(Vec2::new(400.0, 1000.0))
                .with_mask(3)
                .with_friction(1.5)
        )
        .insert(CharacterController::default())
        .with_children(|parent| {
            parent.spawn().insert(Aabb::size(Vec2::new(28.0,28.0)));
        }).id();
    
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
        .with_children(|parent| {
            parent.spawn_bundle((Aabb::size(Vec2::new(600.0, 20.0)),));
        });

    // wall
    commands
    .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(20.0,500.0)),
            material: black.clone(),
            transform : Transform::from_xyz(450.0,0.0,0.0),
            ..Default::default()
    })
    .insert(
        StaticBody2D::new()
    )
    .with_children(|parent| {
        parent.spawn_bundle((Aabb::size(Vec2::new(20.0, 500.0)),));
    });
    // spawn another floor
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(300.0,20.0)),
            material: black.clone(),
            transform : Transform::from_xyz(-300.0,-190.0,0.0),
            ..Default::default()
        })
        .insert(
            StaticBody2D::new()
        )
        .with_children(|parent| {
            parent.spawn_bundle((Aabb::size(Vec2::new(300.0, 20.0)),));
        });

    // yet another floor
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(120.0,20.0)),
            material: black.clone(),
            transform : Transform::from_xyz(360.0,-50.0,0.0),
            ..Default::default()
        })
        .insert(
            StaticBody2D::new()
        )
        .with_children(|parent| {
            parent.spawn_bundle((Aabb::size(Vec2::new(120.0, 20.0)),));
        });

    // is this the last floor?
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(120.0,20.0)),
            material: black.clone(),
            transform : Transform::from_xyz(120.0,-10.0,0.0),
            ..Default::default()
        })
        .insert(
            StaticBody2D::new()
        )
        .with_children(|parent| {
            parent.spawn_bundle((Aabb::size(Vec2::new(120.0, 20.0)),));
        });

    // dude i think there is enough floors already
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(120.0,20.0)),
            material: black.clone(),
            transform : Transform::from_xyz(-120.0,20.0,0.0),
            ..Default::default()
        })
        .insert(
            StaticBody2D::new()
        )
        .with_children(|parent| {
            parent.spawn_bundle((Aabb::size(Vec2::new(120.0, 20.0)),));
        });

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
        )
        .with_children(|parent| {
            parent.spawn_bundle((Aabb::size(Vec2::new(20.0, 20.0)),));
        });

    // spawn the cube connected to us - no longer connected as there are no joints yet
    let _target_id = commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::splat(20.0)),
            material: another_color.clone(),
            transform : Transform::from_xyz(100.0,100.0,0.0),
            ..Default::default()
        })
        .insert(
            KinematicBody2D::new()
                .with_mass(1.0)
        )
        .with_children(|parent| {
            parent.spawn_bundle((Aabb::size(Vec2::new(20.0, 20.0)),));
        }).id();

    // spawn a cube with different layers
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::splat(20.0)),
            material : materials.add(Color::MAROON.into()),
            ..Default::default()
        })
        .insert(
            KinematicBody2D::new()
            .with_layer(2)
            .with_mask(2)
        )
        .with_children(|p| {
            p.spawn().insert(Aabb::size(Vec2::splat(20.0)));
        });

    // spawn the joint between the player and the target cube
    // commands
    //     .spawn()
    //     .insert(
    //         SpringJoint::new(_player_id, _target_id)
    //                     .with_offset(Vec2::new(30.0,30.0))
    //                     .with_rigidness(2.0)
    //     );

    // spawn a sensor square somewhere
    // commands
    //     .spawn_bundle(SpriteBundle {
    //         sprite : Sprite::new(Vec2::new(50.0,50.0)),
    //         material : materials.add(Color::rgba(1.0,0.0,0.0,0.5).into()),
    //         ..Default::default()
    //     })
    //     .insert(RigidBody::new(Mass::Infinite)
    //             .with_sensor(true)
    //             .with_status(Status::Static)
    //             .with_position(Vec2::new(-300.0,-150.0))
    //     );
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
