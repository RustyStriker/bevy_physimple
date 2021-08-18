use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_physimple::{
    physics_components::{velocity::Vel, CollisionLayer},
    prelude::*,
    settings::Gravity,
};

#[derive(Default)]
pub struct CharacterController {
    double_jump : bool,
}

fn main() {
    let mut builder = App::build();
    builder
        .add_plugins(DefaultPlugins)
        .add_plugin(Physics2dPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system());
    builder
        .add_system(character_system.system())
        .add_system(change_sensor_color.system());
    builder.run();
}

fn setup(
    mut commands : Commands,
    mut materials : ResMut<Assets<ColorMaterial>>,
) {
    let blue = materials.add(Color::ALICE_BLUE.into());
    let black = materials.add(Color::BLACK.into());
    let another_color = materials.add(Color::GOLD.into());

    // Spawn the damn camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Spawn character
    let _player_id = commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::splat(28.0)),
            material : blue.clone(),
            // transform : Transform::from_rotation(Quat::from_rotation_z(0.25 * std::f32::consts::PI)),
            ..Default::default()
        })
        .insert_bundle(KinematicBundle {
            obv : Obv {
                offset : Vec2::ZERO,
                shape : BoundingShape::Aabb(Aabb::size(Vec2::splat(28.0))),
            },
            shape: CollisionShape::Square(Square::size(Vec2::splat(28.0))),
            // shape : CollisionShape::Circle(Circle::new(14.0)),
            ..Default::default()
        })
        .insert(CharacterController::default())
        .id();

    // center floor
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(600.0, 30.0)),
            material : black.clone(),
            transform : Transform::from_xyz(150.0, -200.0, 0.0),
            ..Default::default()
        })
        .insert_bundle(StaticBundle {
            shape : CollisionShape::Square(Square::size(Vec2::new(600.0, 30.0))),
            obv : Obv {
                offset : Vec2::ZERO,
                shape : BoundingShape::Aabb(Aabb::size(Vec2::new(600.0, 30.0))),
            },
            coll_layer : CollisionLayer::default(),
        });

    // side wall
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(40.0, 300.0)),
            material : black.clone(),
            transform : {
                let mut t = Transform::from_xyz(450.0, 0.0, 0.0);
                t.rotation = Quat::from_rotation_z(-0.1 * 3.14);
                t
            },
            ..Default::default()
        })
        .insert_bundle(StaticBundle {
            shape : CollisionShape::Square(Square::size(Vec2::new(40.0, 300.0))),
            obv : Obv {
                offset : Vec2::ZERO,
                shape : BoundingShape::Aabb(Aabb::size(Vec2::new(300.0, 300.0))),
            },
            coll_layer : CollisionLayer::default(),
        });

    // smaller other side wall
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::new(30.0, 90.0)),
            material : black.clone(),
            transform : Transform::from_xyz(-150.0, -160.0, 0.0),
            ..Default::default()
        })
        .insert_bundle(StaticBundle {
            shape : CollisionShape::Square(Square::size(Vec2::new(30.0,90.0))),
            obv : Obv {
                offset : Vec2::ZERO,
                shape : BoundingShape::Aabb(Aabb::size(Vec2::new(30.0,90.0)))
            },
            coll_layer : CollisionLayer::default(),
        });

    // Spawn the cube near us
    const CUBE_SIZE : f32 = 40.0;
    commands
        .spawn_bundle(SpriteBundle {
            sprite : Sprite::new(Vec2::splat(CUBE_SIZE)),
            material : another_color.clone(),
            transform : Transform::from_xyz(30.0, -150.0, 0.0),
            ..Default::default()
        })
        .insert(Obv {
            offset : Vec2::ZERO,
            shape : BoundingShape::Aabb(Aabb::size(Vec2::splat(CUBE_SIZE))),
        })
        .insert(CollisionShape::Square(Square::size(Vec2::splat(CUBE_SIZE))))
        .insert(Sensor2D::new());
}

fn character_system(
    input : Res<Input<KeyCode>>,
    time : Res<Time>,
    gravity : Res<Gravity>,
    mut query : Query<(&mut CharacterController, &KinematicBody2D, &mut Vel)>,
) {
    let gravity = gravity.0;

    for (mut controller, body, mut vel) in query.iter_mut() {
        if let Some(normal) = body.on_wall() {
            vel.0 -= normal * 0.1;

            if vel.0.y < -1.0 {
                vel.0.y = -1.0;
            }
        }

        let jump = |body : &KinematicBody2D, vel : &mut Vel| {
            vel.0 = vel.0.slide(gravity.normalize()) - gravity * 0.6;
            let wall = body.on_wall().unwrap_or(Vec2::ZERO) * 250.0;
            vel.0 += wall;
        };

        let should_jump = input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::W);
        if body.on_floor().is_some() || body.on_wall().is_some() {
            controller.double_jump = true;

            if should_jump {
                // This is just a weird way to do jump, using the gravity direction and size(tho you dont need the size)
                // it works by sliding on the gravity direction(so nothing in the direction of gravity)
                // then adding the jump force(here its gravity * 0.5) to the velocity
                jump(body, &mut vel);
            }
        }
        else if controller.double_jump && should_jump {
            controller.double_jump = false;
            jump(body, &mut vel);
        }

        // This is for the testing purpose of the continous collision thingy
        if input.just_pressed(KeyCode::S) && body.on_floor().is_none() {
            vel.0 = Vec2::new(0.0, -50000000.0);
        }

        // It might look like we need to multiply by delta_time but the apply_force function does it for us(in the physics step)
        let acc = Vec2::new(1000.0, 0.0);
        if input.pressed(KeyCode::A) {
            vel.0 -= acc * time.delta_seconds();
            // body.apply_angular_impulse(1.0);
        }
        if input.pressed(KeyCode::D) {
            vel.0 += acc * time.delta_seconds();
            // body.apply_angular_impulse(-1.0);
        }
    }
}

fn change_sensor_color(
    mut materials : ResMut<Assets<ColorMaterial>>,
    q : Query<(&Sensor2D, &Handle<ColorMaterial>)>,
) {
    for (s, h) in q.iter() {
        if let Some(mut m) = materials.get_mut(h) {
            m.color = if s.iter().len() == 0 {
                Color::GOLD
            }
            else {
                Color::rgba(0.0, 0.5, 1.0, 0.5)
            }
        }
    }
}
