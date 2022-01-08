use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_physimple::prelude::*;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "N-body".to_string(),
        width: 600.0,
        height: 600.0,
        vsync: false,
        ..Default::default()
    });

    app
        .add_plugins(DefaultPlugins)
        .add_plugin(Physics2dPlugin);
    
        // FPS in terminal
    app
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        ;

    app
        .add_startup_system(setup.system())
        .add_system(gravity.system())
        .add_system(jumpy.system());

    app.run();
}

fn setup(
    mut coms: Commands,
) {
    // camera
    coms.spawn_bundle(OrthographicCameraBundle::new_2d());

    // do some walls
    let wall_color = Color::BLACK;

    // bottom wall
    coms.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(600.0, 40.0)),
            color: wall_color.clone(),
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, -310.0,0.0),
        ..Default::default()
    })
    .insert_bundle(StaticBundle {
        shape: CollisionShape::Square(Square::size(Vec2::new(600.0,40.0))),
        ..Default::default()
    });

    const SIZE: f32 = 40.0;

    let c1 = Color::RED;
    let c2 = Color::GREEN;

    (0..10).for_each(|i| {
        (0..10).for_each(|k| {
            let c = if (i + k) % 2 == 0 { c1.clone() } else { c2.clone() };
            let pos = Vec2::new(i as f32, k as f32) * SIZE * 1.5 - Vec2::splat(270.0);

            let shape = if (i + k) % 2 == 0 {
                CollisionShape::Square(Square::size(Vec2::splat(SIZE))) // reds
            }
            else {
                CollisionShape::Circle(Circle::new(0.5 * SIZE)) // greens
            };

            coms.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(SIZE)),
                    color: c,
                    ..Default::default()
                },
                transform: Transform::from_translation(pos.extend(0.0)),
                ..Default::default()
            })
            .insert_bundle(KinematicBundle {
                shape: shape,
                ..Default::default()
            });
        })
    })
}

fn gravity(
    time: Res<Time>,
    mut q: Query<&mut Vel>,
) {
    const GRAV: f32 = 420.0;

    for mut v in q.iter_mut() {
        v.0.y -= GRAV * time.delta_seconds();
    }
}
fn jumpy(
    mut q: Query<&mut Vel>,
    mut colls: EventReader<CollisionEvent>,
) {
    const BOUNCE: f32 = 100.0;

    for c in colls.iter() {
        let e = if c.normal.dot(Vec2::Y) > 0.0 { c.entity_a } else { c.entity_b };

        if let Ok(mut v) = q.get_mut(e) {
            v.0.y += BOUNCE
        }
    }
}