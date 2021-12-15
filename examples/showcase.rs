/*
    # Disclaimer!
    I wrote this without actual coding practices and maintainability in mind
    so please dont take any "Coding Lessons" from this file, also it is almost 400 lines

    This is strictly to show what the lib is capable of(and maybe give some people ideas)
    !!! THIS IS NOT TO LEARN FROM !!!
*/

use bevy::prelude::*;
use bevy_physimple::prelude::*;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        width: 1280.0,
        height: 720.0,
        title: "bevy_physimple Showcase".to_string(),
        ..Default::default()
    });

    // plugins
    app
        .add_plugins(DefaultPlugins)
        .add_plugin(Physics2dPlugin)
        ;

    // startup systems
    app
        .add_startup_system(setup.system())
        ;

    // normal systems
    app
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(change_shape.system())
        .add_system(player_movement.system())
        .add_system(sensor_colors.system())
        .add_system(sensor_gravity.system())
        .add_system(ray_head.system())
        ;

    app.run();
}

/// Player marker component
#[derive(Component)]
struct Player;
/// Special gravity for sensors to apply
#[derive(Component)]
struct Gravity(Vec2);
/// Holds colors for the color changing areas
struct ColorChange {
    coll: Handle<ColorMaterial>,
    no_coll: Handle<ColorMaterial>,
}
/// Resource for holding relevant handles, so we wont lose them
struct PlayerHandles {
    capsule_small: Handle<ColorMaterial>,
    square_no_texture: Handle<ColorMaterial>,
    circle: Handle<ColorMaterial>,
}

fn setup(
    mut coms: Commands,
    mut mats: ResMut<Assets<ColorMaterial>>,
    a_server: Res<AssetServer>,
) {
    // Camera
    coms.spawn_bundle(OrthographicCameraBundle::new_2d());

    let  text_style = TextStyle {
        font: a_server.load("fonts/FiraSans-Bold.ttf"),
        font_size: 32.0,
        color: Color::WHITE,
    };
    let text_align = TextAlignment {
        vertical: VerticalAlign::Bottom,
        horizontal: HorizontalAlign::Center,
    };

    // Hello text dump
    coms
        .spawn_bundle(Text2dBundle {
            text: Text::with_section("Hello and Welcome\n\nUse [TAB] to\ncycle between shapes", text_style.clone(), text_align),
            transform: Transform::from_xyz(0.0, 300.0,0.0),
            ..Default::default()
        });

    // PlayerHandles
    let player_capsule = mats.add(a_server.load("capsule_r_25_h_50.png").into());
    let player_circle = mats.add(a_server.load("circle_50_color.png").into());
    let player_square = mats.add(Color::CYAN.into());

    coms.insert_resource(PlayerHandles {
        capsule_small: player_capsule,
        square_no_texture: player_square.clone(), // Cloning this one because it is the default and we want to put it in the player
        circle: player_circle,
    });
    // Player itself :D
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::splat(25.0)),
            material: player_square,
            ..Default::default()
        })
        .insert_bundle(KinematicBundle {
            shape: CollisionShape::Square(Square::size(Vec2::splat(25.0))),
            ..Default::default()
        })
        .insert(Player)
        ;

    // Some objects
    let black = mats.add(Color::BLACK.into());

    // Static text
    coms
        .spawn_bundle(Text2dBundle {
            text: Text::with_section("These are static bodies\nFeel free to\ntest the collisions", text_style.clone(), text_align),
            transform: Transform::from_xyz(300.0, 350.0,0.0),
            ..Default::default()
        });

    // Square static
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::splat(100.0)),
            transform: Transform::from_xyz(300.0, -250.0,0.0),
            material: black.clone(),
            ..Default::default()
        })
        .insert_bundle(StaticBundle {
            shape: CollisionShape::Square(Square::size(Vec2::splat(100.0))),
            ..Default::default()
        })
        ;
    // A nice capsule
    coms
        .spawn_bundle(SpriteBundle {
            material: mats.add(a_server.load("capsule_r_100_h_150.png").into()),
            transform: Transform::from_xyz(300.0, 150.0,0.0),
            ..Default::default()
        })
        .insert_bundle(StaticBundle {
            shape: CollisionShape::Capsule(Capsule::new(50.0, 50.0)),
            ..Default::default()
        })
        ;
    // A (not really) perfect circle
    coms
        .spawn_bundle(SpriteBundle {
            material: mats.add(a_server.load("circle_50.png").into()),
            transform: Transform::from_xyz(300.0,-50.0,0.0),
            ..Default::default()
        })
        .insert_bundle(StaticBundle {
            shape: CollisionShape::Circle(Circle::new(25.0)),
            ..Default::default()
        })
        ;
    
    // Some areas
    
    // Color changer sensor text
    coms
        .spawn_bundle(Text2dBundle {
            text: Text::with_section("This sensor will change\ncolor when you enter it", text_style.clone(), text_align),
            transform: Transform::from_xyz(-350.0,300.0,0.0),
            ..Default::default()
        })
        ;

    // Simple color changer
    let color_changer = ColorChange {
        coll: mats.add(Color::rgba(1.0,1.0,0.0,0.2).into()),
        no_coll: mats.add(Color::rgba(1.0,1.0,0.0,0.7).into()),
    };
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::splat(100.0)),
            material: color_changer.no_coll.clone(),
            transform: Transform::from_xyz(-300.0, 150.0,0.0),
            ..Default::default()
        })
        .insert_bundle(SensorBundle {
            shape: CollisionShape::Square(Square::size(Vec2::splat(100.0))),
            ..Default::default()
        })
        .insert(color_changer)
        ;
    // Gravity thing text dump
    coms
        .spawn_bundle(Text2dBundle {
            text: Text::with_section("This sensor will push\nyou downwards gently\nif you stand in it", text_style.clone(), text_align),
            transform: Transform::from_xyz(-350.0, -210.0,0.0),
            ..Default::default()
        })
        ;
    
    // A neat gravity push
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(100.0,200.0)),
            material: mats.add(Color::rgba(0.5,1.0,0.7,0.3).into()),
            transform: Transform::from_xyz(-300.0, -100.0,0.0),
            ..Default::default()
        })
        .insert_bundle(SensorBundle {
            shape: CollisionShape::Square(Square::size(Vec2::new(100.0,200.0))),
            ..Default::default()
        })
        .insert(Gravity(Vec2::new(0.0,-500.0)))
        ;
    
    // Some text about rays(well, its just rays)
    coms
        .spawn_bundle(Text2dBundle {
            text: Text::with_section("Some RayCasts\nDark Red - Base\nCrimson - Head", text_style, text_align),
            transform: Transform::from_xyz(0.0, -170.0,0.0),
            ..Default::default()
        })
        ;

    // Do some rays and such
    let ray_base = mats.add(Color::rgb(0.5,0.0,0.0).into()); // I want dark red so...
    let ray_head = mats.add(Color::CRIMSON.into()); // The Crimson Scyth
    (0..=30).for_each(|i| {
        let i = (15 - i) as f32; // i32 is kinda useless around here tbh

        coms
            .spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::splat(10.0)),
                material: ray_base.clone(),
                transform: Transform::from_xyz(i * 11.0, -300.0,10.0),
                ..Default::default()
            })
            .insert_bundle(RayCastBundle {
                ray: RayCast::new(Vec2::new(0.0, 150.0)),
                ..Default::default()
            })
            .with_children(|p| {
                p.spawn_bundle(SpriteBundle {
                    sprite: Sprite::new(Vec2::splat(8.0)),
                    material: ray_head.clone(),
                    ..Default::default()
                });
            });
    });
}

fn change_shape(
    p_handles: Res<PlayerHandles>,
    keys: Res<Input<KeyCode>>,
    mut q: Query<(&mut CollisionShape, &mut Handle<ColorMaterial>, &mut Sprite), With<Player>>,
) {
    if let Ok((mut s, mut h, mut sp)) = q.single_mut() {
        // Gonna make a simple change shape kinda thing
        if keys.just_pressed(KeyCode::Tab) {
            match *s {
                CollisionShape::Square(_) => {
                    sp.resize_mode = SpriteResizeMode::Automatic;
                    *h = p_handles.circle.clone();
                    *s = CollisionShape::Circle(Circle::new(25.0));
                },
                CollisionShape::Circle(_) => {
                    *h = p_handles.capsule_small.clone();
                    *s = CollisionShape::Capsule(Capsule::new(25.0, 12.5));
                },
                CollisionShape::Capsule(_) => {
                    sp.resize_mode = SpriteResizeMode::Manual;
                    sp.size = Vec2::splat(25.0);
                    *h = p_handles.square_no_texture.clone();
                    *s = CollisionShape::Square(Square::size(Vec2::splat(25.0)));
                },
                _ => unreachable!(),
            }
        }
    }
}

fn player_movement(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut q: Query<&mut Vel, With<Player>>,
) {
    if let Ok(mut v) = q.single_mut() {
        let mut input = Vec2::ZERO;
        if keys.pressed(KeyCode::W) {
            input.y += 1.0;
        }
        if keys.pressed(KeyCode::S) {
            input.y -= 1.0;
        }
        if keys.pressed(KeyCode::D) {
            input.x += 1.0;
        }
        if keys.pressed(KeyCode::A) {
            input.x -= 1.0;
        }
        input = input.normalize_or_zero();
        // Flat movement
        v.0 = v.0.lerp(input * 200.0, time.delta_seconds() * 5.0);
        v.0 += input * 200.0 * time.delta_seconds();
        // Max speed
        const MAX_SPEED: f32 = 200.0;
        // If you need to check for max speed, checking for `v.0.length_squared() > MAX_SPEED_SQUARED` is usually a better idea
        // Generally try to avoid using `sqrt()` since square root is rather costly
        if v.0.length() > MAX_SPEED {
            v.0 = v.0.normalize_or_zero() * MAX_SPEED; 
        }
    }
}

fn sensor_colors(
    mut q: Query<(&mut Handle<ColorMaterial>, &Sensor, &ColorChange)>,
) {
    for (mut h, s, c) in q.iter_mut() {
        if s.bodies.len() == 0 {
            *h = c.no_coll.clone();
        }
        else {
            *h = c.coll.clone();
        }
    }
}

fn sensor_gravity(
    time: Res<Time>,
    mut vels: Query<&mut Vel>,
    q: Query<(&Sensor, &Gravity)>
) {
    for (s, g) in q.iter() {
        for &e in s.bodies.iter() {
            if let Ok(mut v) = vels.get_mut(e) {
                v.0 += g.0 * time.delta_seconds();
            }
        }
    }
}

fn ray_head(
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