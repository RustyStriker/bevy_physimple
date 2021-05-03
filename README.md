# physimple

Physimple aims to be the simplest(and capable) physics engine(currently for bevy)

## Why?

Because I love physics and I love programming, so what is better? physics programming!
besides, simulation physics can be restricting when you want to do some weird physics behaviour for games,
and eventually i want people to be able to use this crate as a simple collisions solver.

## What is currently working?

- Simple AABB collisions(without rotations)
- That is pretty much it

## Quickstart

TODO, for now you can check the examples
just run
`cargo run --bevy/dynamic --example (showcase/simple/stress_2d) --release`
to see it in action

## bevy - physimple versions

| bevy | physimple       |
|------|-----------------|
| 0.5  | 0.1.0 - current |

## planned

- [ ] Rotation
- [ ] More collision shapes
- [x] Better friction - with per object values
- [x] Raycasts
- [ ] Joints(with support for custom types of joints)
