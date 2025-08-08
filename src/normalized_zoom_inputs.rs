use bevy::input::{
    gestures::PinchGesture,
    mouse::{MouseScrollUnit, MouseWheel},
};
use bevy::prelude::*;

/// Holds normalized zoom inputs constructed from
/// raw events.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NormalizedZoomInputs {
    pub pinch: f32,
    pub wheel: f32,
}

impl NormalizedZoomInputs {
    /// Reads [`MouseWheel`] and [`PinchGesture`] [`MessageReader`]s, normalizes
    /// them, and returns a new [`NormalizedZoomInputs`].
    pub(crate) fn from_events(
        mut mouse_wheel_events: MessageReader<MouseWheel>,
        mut pinch_gesture_events: MessageReader<PinchGesture>,
    ) -> Self {
        let pinch = pinch_gesture_events.read().map(|ev| ev.0).sum::<f32>();

        // Wheel event deltas are roughly 1000 larger scale than equivalent
        // pinchs and way too large to apply as-is, so we normalize them by
        // dividing by 1000.
        const MOUSE_WHEEL_NORMALIZE_FACTOR: f32 = 0.001;
        const PIXELS_PER_LINE: f32 = 100.; // Maybe make configurable?
        let wheel = mouse_wheel_events
            .read()
            .map(|ev| match ev.unit {
                MouseScrollUnit::Pixel => ev.y,
                MouseScrollUnit::Line => ev.y * PIXELS_PER_LINE,
            })
            .sum::<f32>()
            * MOUSE_WHEEL_NORMALIZE_FACTOR;

        Self { pinch, wheel }
    }

    /// Apply sensitivity scalers to the inputs and return a final zoom delta
    /// to apply.
    pub(crate) fn apply_sensitivity(&self, wheel_sensitivity: f32, pinch_sensitivity: f32) -> f32 {
        self.pinch * pinch_sensitivity + self.wheel * wheel_sensitivity
    }

    /// True when no input.
    pub(crate) fn is_empty(self) -> bool {
        self.pinch == 0. && self.wheel == 0.
    }
}
