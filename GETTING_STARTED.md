# Getting Started

The plugin's name is `Physics2dPlugin` so in order to use it you need to do `App.add_plugin(Physics2dPlugin);`

The plugin contains the following components and bundles(with a brief explanation):

- `CollisionShape`: Enum which holds the collision shape
- `KinematicBundle`(bundle): Contains the needed components for a continuous collision KinematicBody
- `StaticBundle`(bundle): Contains the needed components for a StaticBody
- `StaticBody`: Marker component, StaticBody V StaticBody/Sensor collisions cannot occur
- `SensorBundle`(bundle): Contains the needed components for a Sensor
- `Sensor`: Marker component, but also holds information about the colliding bodies in a Vec(might be changed in favor of events/hash sets)
- `RayCastBundle`(bundle): Contains the needed components for a RayCast
- `RayCast`: Gets the closest collision occurring on a given ray
- `CollisionLayer`: Which collision layer and mask the body occupies(a collision can occur only if `a.mask & b.layer | a.layer & b.mask != 0`)
- `Vel`: Used for Continuous collision kinematic bodies, requires more computational power, so not a good idea for small visual particles(like debris), yet good for stuff like bullets
- `Transform2D`: Used internally, if you are modifying the position/rotation of an object during a physics step, it's better to modify this component instead.

You may also use the following events:

- `CollisionEvent`
- more will probably come in the future(feel free to suggest events)

And of course, the following resource:

- `TransformMode`: Allows you to pick which 2D plane you want to "project" your physics on

This lib takes care of:

- collision
- solving said collision and providing some information about them
- that is pretty much it

while it also provides continuous collision, it is quite limited and works only for `With<Vel>` against entities marked with `StaticBody` or `Sensor`

What you need to take care of:

- Gravity(If you want)
- Applying movement(except for `With<Vel>` entities)
- Actually reacting to the collision events(solving is done automatically, and `With<Vel>` will slide the movement along the collision normal)

Now I know you might be asking yourself:

```plain
But why do I have to take care of all those stuff?
shouldn't the physics engine take care of it???
???
```

and for that my friend, let me inform you of my goal here,
this is my attempt at creating a "minimalistic" physics engine,
so technically its mostly collision detection and solving(and even that is not that good tbh)

The reason is, games can have funky and different physics,
whether it's a top-down shooter with rigid controls,
or a game which attempts at mimicking `Titanfall 2`'s movement(please make one, even just a demo),
games have a lot of unrealistic physics, because real physics ain't always fun(you can't double jump in real life).

I am getting derailed here... Gonna finish this rant some when and move it to somewhere more appropriate.

## Actually using it

To make a minimalist example, we first need to add the plugin in main, so make sure you are doing:

```rs
app.add_plugin(Physics2dPlugin);
```

Now we can spawn some physics objects.

So in our startup system we are going to add some physical bodies with sprites:

```rs
fn startup(
    mut coms: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn a camera in case we didnt add 1 already
    coms.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Spawn a floor
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(600.0, 30.0)),
            material: materials.add(Color::BLACK.into()),
            transform: Transform::from_xyz(150.0, -200.0, 0.0),
            ..Default::default()
        }) // The sprite bundle already inserts the `Global/Transform` components
        .insert_bundle(StaticBundle {
            marker: StaticBody, // This is an empty struct
            shape: CollisionShape::Square(Square::size(Vec2::new(600.0, 30.0))),
            coll_layer: CollisionLayer::default(),
        })
        ;
    
    // And we gonna spawn a simple cube using continuous collision
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::splat(35.0)),
            material: another_color.clone(),
            ..Default::default()
        })
        .insert_bundle(KinematicBundle {
            shape: CollisionShape::Square(Square::size(Vec2::splat(35.0))),
            ..Default::default()
        })
        ;
    // Spawn another cube without contuous collision
    coms
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::splat(35.0)),
            material: another_color.clone(),
            ..Default::default()
        })
        .insert(CollisionShape::Square(Square::size(Vec2::splat(35.0)))) // The collision shape
        .insert(CollisionLayer::default()) // And collision layers
        ;
    // make sure not to have `Sensor/StaticBody` as it will turn this body
    // into a Sensor/Staticbody instead.
}
```

### NOTE

All physical bodies need both `Transform` and `GlobalTransform` to work,
but they are not a part of the given bundles,
as I assumed you will only use them with a `SpriteBundle` or something else which already holds a `Transform` + `GlobalTransform` with it.

(You can create your own bundles quite easily, as they hold 3 components each)

For a more "full" example, please check the only existing example currently named `shapes`

### NOTE — 2

If you are using non-continuous collision kinematic bodies(`Without<Vel>`),
and you apply gravity to them, you will need to read the `CollisionEvent`s
and slide(or reflect/bounce/whatever) the movement along the collision normal
to prevent the bodies from endlessly accelerating downwards,
eventually causing them to be too fast for `Without<Vel>` to handle.

Lastly, if you have any questions feel free to @ me on the bevy discord.
