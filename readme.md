# bevy-pancam

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



## Issues

Zoom doesn't work on bevy 0.5 unless you cherry-pick this patch: https://github.com/bevyengine/bevy/pull/2015