#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

pub mod app;

mod consts;
mod renderer;
mod sdl2_renderer;
mod vec2;
mod world;
