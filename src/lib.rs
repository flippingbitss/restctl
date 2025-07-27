#![warn(clippy::all, rust_2018_idioms)]
mod app;
mod code;
mod styles;
mod text_edit;
pub use app::App;
pub use styles::customize_app_styles;
