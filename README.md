# bevy_pancam

[![crates.io](https://img.shields.io/crates/v/bevy_pancam.svg)](https://crates.io/crates/bevy_pancam)
![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![crates.io](https://img.shields.io/crates/d/bevy_pancam.svg)](https://crates.io/crates/bevy_pancam)
[![docs.rs](https://img.shields.io/docsrs/bevy_pancam)](https://docs.rs/bevy_pancam)

A 2d camera plugin for bevy that works with orthographic cameras.

The motivation is that this could be used for something like a map editor for a 2d game.

## Controls

Behaves similarly to common online map applications:

- Click and drag to move the camera
- Scroll to zoom
- Hold keyboard buttons to move the camera

## Usage

Add the plugin to your app

```rust ignore
App::new()
    .add_plugins((DefaultPlugins, PanCamPlugin::default()))
    .run();
```

Add the component to an orthographic camera:

```rust ignore
commands.spawn((
    Camera2d,
    PanCam::default(),
))
```

This is enough to get going with sensible defaults.

Alternatively, set the fields of the `PanCam` component to customize behavior:

```rust ignore
commands.spawn((
    Camera2d,
    PanCam {
        grab_buttons: vec![MouseButton::Left, MouseButton::Middle], // which buttons should drag the camera
        move_keys: DirectionKeys {      // the keyboard buttons used to move the camera
            up:    vec![KeyCode::KeyQ], // initalize the struct like this or use the provided methods for
            down:  vec![KeyCode::KeyW], // common key combinations
            left:  vec![KeyCode::KeyE],
            right: vec![KeyCode::KeyR],
        },
        speed: 400., // the speed for the keyboard movement
        enabled: true, // when false, controls are disabled. See toggle example.
        zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
        min_scale: 1., // prevent the camera from zooming too far in
        max_scale: 40., // prevent the camera from zooming too far out
        min_x: f32::NEG_INFINITY, // minimum x position of the camera window
        max_x: f32::INFINITY, // maximum x position of the camera window
        min_y: f32::NEG_INFINITY, // minimum y position of the camera window
        max_y: f32::INFINITY, // maximum y position of the camera window
    },
));
```

See the [`simple`](./examples/simple.rs) and [`toggle`](./examples/toggle.rs) examples.

## Cargo features

- `bevy_egui` makes pancam cameras not react when the mouse or keyboard focus is on widgets created with [`bevy_egui`](https://github.com/mvlabat/bevy_egui)

## Bevy Version Support

The `main` branch targets the latest bevy release.

|bevy|bevy_pancam|
|----|-----------|
|0.15|0.16, 0.17, main |
|0.14|0.12, 0.13, 0.14, 0.15|
|0.13|0.11       |
|0.12|0.10       |
|0.11|0.9        |
|0.10|0.8        |
|0.9 |0.7,       |
|0.8 |0.5, 0.6   |
|0.7 |0.3, 0.4   |
|0.6 |0.2        |
|0.5 |0.1        |

## License

`bevy_pancam` is dual-licensed under either

- MIT License (./LICENSE-MIT or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 (./LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Contributions

PRs welcome!
