# Bevy_physimple

Bevy_physimple is a collision detection(and solver) plugin for the bevy game engine.

## Current state

I keep updating it wwith every bevy update, and  will probably continue to
do so for as long as I can spare the time for it(which is for quite some time currently),
but I shifted focus to making/trying to make games in my free time,
so the big stuff I need/want to do will have to wait for now.

## Why?

Because I love physics and I love programming, so what is better? Physics programming!
Besides, simulation physics can be restricting when you want to do some weird physics behavior for games,
and eventually I want people to be able to use this crate as a simple collision solver.

## What is currently working?

- Square, Circle, Capsule and custom collision shapes
- Sensors, Static and normal kinematic bodies
- Rays

## What doesn't work/is currently buggy?

- Continuous collision is now disabled and awaiting a rewrite
- Scale doesn't affect the shapes
- You can push objects through walls, if the wall is too thin the object might tunnel through it
- Probably some more stuff, please tell me when something isn't working properly(and isn't written here, or has an issue)

## Quickstart

Clone the repo, and run

    cargo run --example showcase --release

Or check out the `GETTING_STARTED.md` file.

## Bevy — physimple versions

| bevy | physimple       |
|------|-----------------|
| 0.5  | 0.1.0 — 0.2.0   |
| 0.6  | 0.3.0           |
| 0.7  | 0.4.0           |
| 0.8  | 0.5.0 - current |

## Features todo list

- [x] Better manual ray casting support
- [x] Support multiple shapes on the same object(now possible with `CollisionShape::Multiple`)
- [ ] Better/Rewrite continuous collision
- [ ] Make continuous collision (fully) optional
- [ ] Start adding 3D support
- [ ] Add bugs to fix
- [ ] A new simple platformer game example
- [ ] A new simple top down game example
