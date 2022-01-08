use bevy::{math::Mat2, prelude::*};
use bevy_physimple::prelude::*;

fn main() {
    let mut app = App::new();

    app
        .add_plugins(DefaultPlugins)
        .add_plugin(Physics2dPlugin)
        .add_system(bevy::input::system::exit_on_esc_system.system());

    app
        .add_startup_system(setup.system())
        .add_system(move_controller.system());

    app.run();
}

struct MyTriangle {
    v1: Vec2,
    v2: Vec2,
    v3: Vec2
}
impl SAT for MyTriangle {
    fn get_normals(&self, trans: &Transform2D) -> Vec<Vec2> {
        let rot = Mat2::from_angle(trans.rotation());
        let edges = [
            rot * (self.v1 - self.v2),
            rot * (self.v2 - self.v3),
            rot * (self.v3 - self.v1)
        ];
        
        edges.iter().map(|&e| e.normalize().perp()).collect()
    }

    fn project(&self, trans: &Transform2D, normal: Vec2) -> (f32,f32) {
        let rot = Mat2::from_angle(trans.rotation());
        
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;

        for v in [self.v1,self.v2,self.v3] {
            let v = rot * v + trans.translation();
            let proj = v.dot(normal);

            min = min.min(proj);
            max = max.max(proj);
        }
        (min,max)
    }

    fn get_closest_vertex(&self, trans: &Transform2D, vertex: Vec2) -> Vec2 {
        let rot = Mat2::from_angle(trans.rotation());

        let mut c = f32::INFINITY;
        let mut p = Vec2::ZERO;
        
        for v in [self.v1,self.v2,self.v3] {
            let v = rot * v + trans.translation();
            let d = (v - vertex).length_squared();
            if d < c {
                c = d;
                p = v;
            }
        }
        p
    }

    fn ray(&self, _: &Transform2D, _: Vec2, _:  Vec2) -> Option<f32> {
        // Doesnt matter for normal collision, but it will break continuous collision and RayCast against this shape
        None
    }
}

#[derive(Component)]
struct Controller;

fn setup(
    mut coms: Commands,
    asset_server: Res<AssetServer>,
) {
    // camera
    coms.spawn_bundle(OrthographicCameraBundle::new_2d());

    // triangle
    coms.spawn_bundle(SpriteBundle {
        texture: asset_server.load("triangle.png"),
        ..Default::default()
    })
    .insert_bundle(StaticBundle {
        marker: StaticBody,
        shape: CollisionShape::Convex(Box::new(MyTriangle {
            v1: Vec2::new(-100.0,100.0),
            v2: Vec2::new(100.0,100.0),
            v3: Vec2::new(0.0,-100.0),
        })),
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

fn move_controller(
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