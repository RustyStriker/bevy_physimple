# physimple

Physimple aims to be the simplest(and capable) physics engine(currently for bevy)

## WARNING

Beware for breaking changes with each update for now, as I am trying to make stuff work better by adding/removing stuff

## Why?

Because I love physics and I love programming, so what is better? physics programming!
besides, simulation physics can be restricting when you want to do some weird physics behaviour for games,
and eventually i want people to be able to use this crate as a simple collisions solver.

## What is currently working?

- Square, Circle, Capsule and custom collision shapes
- Sensors and Staticbodies
- Continuous collision

## What doesnt work/What is currently buggy?

- Scale doesnt affect the shapes
- You can push objects through walls, if the wall is too thin the object might tunnle through it
- Sometimes if a cube is fast enough and under the right conditions, it will tunnle through(can be noticed in the `platformer` example, tho shouldn't affect normal usuage)
- Probably some more stuff, please tell me when something isn't working properly(and isn't written here, or has an issue)

## Quickstart

Clone the repo, and run

    cargo run --example showcase --release

Or check out the `GETTING_STARTED.md` file.

## bevy - physimple versions

| bevy | physimple       |
|------|-----------------|
| 0.5  | 0.1.0 - current |

## 0.2.0 Todo list

- [x] Rewrite shape overlap
- [x] Implement the different shapes
  - [x] Capsule - Rays
  - [x] Circle - Rays
  - [x] Generic shape(dynamic object)
- [x] Update the examples, and better showcase stuff
  - [x] n-body example(somewhat n-body)
  - [x] General showcase(needs review)
  - [x] Convex shapes
- [x] Rotate offsets(and go through everything making sure rotations are included)
- [ ] Actual docs(this might never be enough)
