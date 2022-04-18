# bevy_pancam

[![crates.io](https://img.shields.io/crates/v/bevy_pancam.svg)](https://crates.io/crates/bevy_pancam)
![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![crates.io](https://img.shields.io/crates/d/bevy_pancam.svg)](https://crates.io/crates/bevy_pancam)
[![docs.rs](https://img.shields.io/docsrs/bevy_pancam)](https://docs.rs/bevy_pancam)

A 2d-camera plugin for bevy that works with orthographic cameras.

The motivation is that this could be used for something like a map editor for a 2D game.

## Controls

Behaves similarly to common online map applications:

- Click and drag to move the camera
- Scroll to zoom

## Usage

Add the plugin to your app

```rust
App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(PanCamPlugin::default());
```

```rust
commands.spawn_bundle(OrthographicCameraBundle::new_2d())
    .insert(PanCam::default());
```

See the [`simple`](./examples/simple.rs) example.

## Bevy Version Support

The `main` branch targets the latest bevy release.

I intend to support the `main` branch of Bevy in the `bevy-main` branch.

|bevy|bevy_pancam|
|---|---|
|0.7|0.3, main|
|0.6|0.2|
|0.5|0.1|

## License

MIT or Apache-2.0

## Contributions

PRs welcome!