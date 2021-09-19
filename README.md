# physimple

Physimple aims to be the simplest(and capable) physics engine(currently for bevy)

## WARNING

Beware for breaking changes with each update for now, as I am trying to make stuff work better by adding/removing stuff

## Why?

Because I love physics and I love programming, so what is better? physics programming!
besides, simulation physics can be restricting when you want to do some weird physics behaviour for games,
and eventually i want people to be able to use this crate as a simple collisions solver.

## What is currently working?

- Square and Circle collision shapes
- Sensors and Staticbodies
- Continuous collision

## What doesnt work/What is currently buggy?

- Raycasts with circles
- Overlap detection sometimes acts weird(apperant in Sensor vs Kinematic when the kinematic isn't moving)
- You can push objects through walls, if the wall is too thin the object might tunnle through it
- Probably some more stuff, please tell me when something isn't working properly(and isn't written here)

## Quickstart

Check out the `GETTING_STARTED.md` file in the base of the repo(warning, WIP like the rest of this lib).

## bevy - physimple versions

| bevy | physimple       |
|------|-----------------|
| 0.5  | 0.1.0 - current |

## 0.2.0 Todo list

- [x] Rewrite shape overlap
- [ ] Implement the different shapes
  - [ ] Capsule
  - [ ] Generic shape(dynamic object)
- [ ] Update the examples, and better showcase stuff
- [ ] Basic joints
- [ ] Actual docs
