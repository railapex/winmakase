//! Rust port of the winsome TypeScript theme adapter (`adapter/src/`): parses an omarchy
//! v4.0.1 `colors.toml` into a validated `Palette`, resolves light/dark mode, and renders a
//! Windows Terminal color scheme. See the TS originals for the spec this mirrors.

pub mod error;
pub mod light_dark;
pub mod palette;
pub mod render;

pub use error::ThemeError;
pub use light_dark::{is_light_theme, relative_luminance};
pub use palette::{Mode, Palette, load_palette, parse_palette};
