use bevy::prelude::*;
use bevy_physimple::prelude::*;

fn main() {
    let mut app = App::new();

    app
        .add_plugins(DefaultPlugins)
        .add_plugin(Physics2dPlugin)
        .add_system(bevy::window::close_on_esc);

    app
        .add_startup_system(setup_sys)
        .add_system(move_controller_sys);

    app.run();
}

#[derive(Component)]
struct Controller;

fn setup_sys(
    mut coms: Commands,
    asset_server: Res<AssetServer>,
) {
    // camera
    coms.spawn_bundle(Camera2dBundle::default());

    // triangle
    coms.spawn_bundle(SpriteBundle {
        texture: asset_server.load("capsule_r_100_h_150.png"),
        ..Default::default()
    })
    .insert_bundle(StaticBundle {
        marker: StaticBody,
        // The texture's dimension are indeed 100x150, but the height is the distance between the 2 centers of the edge circles
        // thus we need to do `height = acutal_size(150) - 2 * radius(100) = 50`
        shape: CollisionShape::Capsule(Capsule::new(50.0,50.0)),
        coll_layer: CollisionLayer::default(),
    });

    // spawn a moveable player
    coms.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::splat(30.0)),
            color: Color::MIDNIGHT_BLUE,
            ..Default::default()
        },
        transform: Transform::from_xyz(150.0,150.0,0.0),
        ..Default::default()
    })
    // this is pretty much how you get a non continuous collision kinematic object
    .insert(CollisionShape::Square(Square::size(Vec2::splat(30.0))))
    .insert(CollisionLayer::default())
    .insert(Controller);
}

fn move_controller_sys(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut q: Query<&mut Transform, With<Controller>>,
) {
    for mut t in q.iter_mut() {
        let mut movement = Vec2::ZERO;

        if keyboard.pressed(KeyCode::W) {
            movement.y += 1.0;
        }
        if keyboard.pressed(KeyCode::S) {
            movement.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::D) {
            movement.x += 1.0;
        }
        if keyboard.pressed(KeyCode::A) {
            movement.x -= 1.0;
        }

        t.translation += movement.extend(0.0) * time.delta_seconds() * 100.0;
    }
}