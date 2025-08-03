#![warn(clippy::all, rust_2018_idioms)]
mod app;
pub mod async_runtime;
mod auth;
mod components;
mod core;
mod header;
mod http;
mod styles;
pub mod tasks;
mod tiles;
pub use app::App;
pub use styles::customize_app_styles;
