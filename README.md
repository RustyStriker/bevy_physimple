# physimple

Physimple aims to be the simplest(and capable) physics engine(currently for bevy)

## WARNING

Beware for breaking changes with each update for now, as I am trying to make stuff work better by adding/removing stuff

## Why?

Because I love physics and I love programming, so what is better? physics programming!
besides, simulation physics can be restricting when you want to do some weird physics behaviour for games,
and eventually i want people to be able to use this crate as a simple collisions solver.

## What is currently working?

- The Sqaure collision shape, tho it acts weird due to bad overlap detection code.

## Quickstart

TODO, for now you can check the examples
just run
`cargo run --bevy/dynamic --example shapes --release`
to see it in action(other examples are not updated to use shapes and will not work currently)

Do note that each physics component needs a `GlobalTransform` with it now,
and `KinematicBody2D` needs a `Transform` in order to actually move and be used.

## bevy - physimple versions

| bevy | physimple       |
|------|-----------------|
| 0.5  | 0.1.0 - current |

## A todo list

- [ ] Rewrite shape overlap - current
- [ ] Implement the different shapes
- [ ] Update the examples, and better showcase stuff
