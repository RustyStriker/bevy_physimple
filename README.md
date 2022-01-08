# Bevy_physimple

Bevy_physimple is a collision detection(and solver) plugin for the bevy game engine.

## Why?

Because I love physics and I love programming, so what is better? Physics programming!
Besides, simulation physics can be restricting when you want to do some weird physics behavior for games,
and eventually I want people to be able to use this crate as a simple collision solver.

## What is currently working?

- Square, Circle, Capsule and custom collision shapes
- Sensors, Static and normal kinematic bodies
- Continuous collision
- Rays

## What doesn't work/is currently buggy?

- Scale doesn't affect the shapes
- You can push objects through walls, if the wall is too thin the object might tunnel through it
- Sometimes if a cube is fast enough and under the right conditions, it will tunnel through(can be noticed in the `platformer` example, tho shouldn't affect normal usage)
- Probably some more stuff, please tell me when something isn't working properly(and isn't written here, or has an issue)

## Quickstart

Clone the repo, and run

    cargo run --example showcase --release

Or check out the `GETTING_STARTED.md` file.

## Bevy — physimple versions

| bevy | physimple       |
|------|-----------------|
| 0.5  | 0.1.0 — 0.2.0   |
|------|-----------------|
| 0.6  | 0.3.0 — current |

## 0.4.0

- [ ] Even more docs(Honestly, if something is missing just open an issue in github/@ me in the bevy discord/dm me)
- [ ] Optimize Broad Phase with cool and flashy algorithms(I dont think those said algorithms will actually help that much)
- [x] Better manual ray casting support
- [ ] A new simple platformer game example
- [ ] A new simple top down game example
