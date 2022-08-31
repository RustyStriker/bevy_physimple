use bevy::prelude::*;
use bevy_physimple::prelude::*;

#[derive(Default, Component)]
pub struct Player {
    double_jump: bool,
    on_wall: Option<Vec2>,
    on_floor: bool
}

pub struct Gravity(Vec2);

fn main() {
    let mut app = App::new();
    app // Basic setup of the app
        .insert_resource(WindowDescriptor {
            title: "A cool name for an example".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Physics2dPlugin)
        .add_system(bevy::window::close_on_esc)
        ;
    app // startup systems
        .add_startup_system(setup_sys)
        ;
    app // systems
        .add_system(controller_on_stuff_sys)
        .add_system(character_system_sys)
        .add_system(change_sensor_color_sys)
        .add_system(gravity_sys)
        .add_system(ray_head_sys)
        .add_system(move_player_sys)
        .add_system(slide_movement_sys)
        ;
    app.run();
}

fn setup_sys(
    mut coms: Commands,
    a_server: Res<AssetServer>,
) {
    let wall = Color::BLACK;

    // insert a gravity struct
    coms.insert_resource(Gravity(Vec2::new(0.0,-540.0)));

    // Spawn the damn camera
    coms.spawn_bundle(Camera2dBundle::default());
    
    // Controls
    let style = TextStyle {
        font: a_server.load("fonts/FiraSans-Bold.ttf"),
        font_size: 32.0,
        color: Color::ANTIQUE_WHITE,
    };
    let text = "A/D - Movement\nSpace/W - Jump/Double jump\nS - Stomp(when mid air)";
    coms
        .spawn_bundle(Text2dBundle {
            text: Text::from_section(text, style),
            transform: Transform::from_xyz(-270.0, 360.0, 0.0),
            ..Default::default()
        })
        ;

    // Spawn character
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(28.0)),
                color: Color::ALICE_BLUE,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle(KinematicBundle {
            shape: CollisionShape::Square(Square::size(Vec2::splat(28.0))),
            ..Default::default()
        })
        .insert(Player::default())
        .insert(
        RayCast::new(Vec2::new(100.0,0.0))
            .with_offset(Vec2::new(14.0,0.0))     // Gonna offset our ray
            .with_static(true)       // Let it collide with static bodies
        )
        .with_children(|p| {
            // We gonna push a little cube for the ray's head
            p.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(10.0)),
                    color: Color::MIDNIGHT_BLUE,
                    ..Default::default()
                },
                ..Default::default()
            });
        })
        ;

    // center floor
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(600.0, 30.0)),
                color: wall,
                ..Default::default()
            },
            transform: Transform::from_xyz(150.0, -200.0, 0.0),
            ..Default::default()
        })
        .insert_bundle(StaticBundle {
            shape: CollisionShape::Square(Square::size(Vec2::new(600.0, 30.0))),
            ..Default::default()
        })
        ;

    // side wall
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(40.0, 300.0)),
                color: wall,
                ..Default::default()
            },
            transform: {
                let mut t = Transform::from_xyz(450.0, 0.0, 0.0);
                t.rotation = Quat::from_rotation_z(-0.1 * 3.14);
                t
            },
            ..Default::default()
        })
        .insert_bundle(StaticBundle {
            shape: CollisionShape::Square(Square::size(Vec2::new(40.0, 300.0))),
            ..Default::default()
        })
        ;

    // smaller other side wall
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(30.0, 90.0)),
                color: wall,
                ..Default::default()
            },
            transform: Transform::from_xyz(-150.0, -160.0, 0.0),
            ..Default::default()
        })
        .insert_bundle(StaticBundle {
            shape: CollisionShape::Square(Square::size(Vec2::new(30.0,90.0))),
            ..Default::default()
        })
        ;
    
    // Floating platform
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(200.0,30.0)),
                color: wall,
                ..Default::default()
            },
            transform: Transform::from_xyz(-150.0, 0.0,0.0),
            ..Default::default()
        })
        .insert_bundle(StaticBundle {
            shape: CollisionShape::Square(Square::size(Vec2::new(200.0, 30.0))),
            ..Default::default()
        })
        ;

    // Spawn the sensor
    const SENSOR_SIZE: f32 = 50.0;
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(SENSOR_SIZE)),
                color: Color::GOLD,
                ..Default::default()
            },
            transform: Transform::from_xyz(30.0, -150.0, 0.0),
            ..Default::default()
        })
        .insert_bundle(SensorBundle {
            shape: CollisionShape::Square(Square::size(Vec2::splat(SENSOR_SIZE))),
            ..Default::default()
        });

    // Spawn another cube which we will try to push or something
    const CUBE_SIZE: f32 = 35.0;
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(CUBE_SIZE)),
                color: Color::CRIMSON,
                ..Default::default()
            },
            transform: Transform::from_xyz(100.0,0.0,0.0),
            ..Default::default()
        })
        .insert_bundle(KinematicBundle {
            shape: CollisionShape::Square(Square::size(Vec2::splat(CUBE_SIZE))),
            ..Default::default()
        })
        ;
}

// We need this system since Vel is currently disabled internally
fn move_player_sys(
    time: Res<Time>,
    mut q: Query<(&Vel, &mut Transform)>
) {
    for (v, mut t) in q.iter_mut() {
        t.translation += v.0.extend(0.0) * time.delta_seconds();
    }
}

pub fn slide_movement_sys(
    mut coll_events: EventReader<CollisionEvent>,
    mut query: Query<&mut Vel>,
) {
    for c in coll_events.iter() {
        if let Ok(mut v) = query.get_mut(c.entity_a) {
            if v.0.dot(c.normal) < 0.0 {
                v.0 = v.0.slide(c.normal);
            }
        }
        if let Ok(mut v) = query.get_mut(c.entity_b) {
            if v.0.dot(-c.normal) < 0.0 {
                v.0 = v.0.slide(c.normal);
            }
        }
    }
}

fn gravity_sys(
    time: Res<Time>,
    grav: Res<Gravity>,
    mut q: Query<&mut Vel>,
) {
    // Since the lib itself doesnt take care of gravity(for obv reasons) we need to do it here
    let g = grav.0;
    let t = time.delta_seconds();

    for mut v in q.iter_mut() {
        v.0 += t * g;
    }
}

fn controller_on_stuff_sys(
    mut query: Query<(Entity, &mut Player)>,
    mut colls: EventReader<CollisionEvent>,
) {
    // Iterate over the collisions and check if the player is on a wall/floor
    let (e, mut c) = query.single_mut();

    // clear the current data on c
    c.on_floor = false;
    c.on_wall = None;

    for coll in colls.iter().filter(|&c| c.is_b_static) {
        if coll.entity_a == e {
            let n = coll.normal.dot(Vec2::Y);

            if n > 0.7 {
                c.on_floor = true;
            }
            else if n.abs() <= 0.7 {
                c.on_wall = Some(coll.normal);
            }
        }
    }
}

fn character_system_sys(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    gravity: Res<Gravity>,
    mut query: Query<(&mut Player, &mut Vel)>,
) {
    let gravity = gravity.0;

    for (mut controller, mut vel) in query.iter_mut() {
        if let Some(normal) = controller.on_wall {
            // If we are colliding with a wall, make sure to stick
            vel.0 -= normal * 0.1;
            // and limit our speed downwards
            if vel.0.y < -1.0 {
                vel.0.y = -1.0;
            }
        }
        // There are 2 places in which we apply a jump, so i made a little colsure for code reusability
        let jump = |body: &Player, vel: &mut Vel| {
            vel.0 = vel.0.slide(gravity.normalize()) - gravity * 0.6;
            let wall = body.on_wall.unwrap_or(Vec2::ZERO) * 250.0;
            vel.0 += wall;
        };

        let should_jump = input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::W);
        if controller.on_floor || controller.on_wall.is_some() {
            controller.double_jump = true;
            if should_jump {
                jump(&controller, &mut vel);
            }
        }
        else if controller.double_jump && should_jump {
            controller.double_jump = false;
            jump(&controller, &mut vel);
        }

        // This is for the testing purpose of the continuous collision - aka "The Stomp"
        if input.just_pressed(KeyCode::S) && !controller.on_floor {
            vel.0 = Vec2::new(0.0, -5000.0);
        }
        // REMINDER: Dont forget to multiply by `time.delta_seconds()` when messing with movement
        let acc = Vec2::new(1000.0, 0.0) * time.delta_seconds();
        if input.pressed(KeyCode::A) {
            vel.0 -= acc;
        }
        else if input.pressed(KeyCode::D) {
            vel.0 += acc;
        }
        else {
            // This is not a good way to do friction
            vel.0.x *= 1.0 - (10.0 * time.delta_seconds());
        }

        // terminal velocity
        const TERMINAL_X: f32 = 500.0;
        if vel.0.x.abs() > TERMINAL_X {
            vel.0.x = TERMINAL_X.copysign(vel.0.x); // you can also do `TERMINAL_X * vel.0.x.signum()`
        }
        
    }
}

fn change_sensor_color_sys(
    mut q: Query<(&Sensor, &mut Sprite)>,
) {
    // Simply change the color of the sensor if something is inside it
    for (s, mut h) in q.iter_mut() {
        h.color = if s.bodies.len() == 0 {
            Color::GOLD
        }
        else {
            Color::rgba(0.0, 0.5, 1.0, 0.5)
        }
    }
}

fn ray_head_sys(
    mut ts: Query<&mut Transform, Without<RayCast>>,
    q: Query<(&RayCast, &Children, &Transform)>,
) {
    for (r,c, rt) in q.iter() {
        if let Some(c) = c.first() {
            if let Ok(mut t) = ts.get_mut(*c) {
                // We use the offset in the `unwrap_or` because we want to offset the position to be where the ray "ends"
                // while in the `map`(and `pos` by extension) we want the position relative to the transform component
                // since `a.collision_point` is in global space

                let pos = Vec2::new(rt.translation.x, rt.translation.y);
                t.translation = r.collision.map(|a| a.collision_point - pos).unwrap_or(r.cast + r.offset).extend(0.0);
            }
        }
    }
}