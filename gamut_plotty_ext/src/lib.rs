#![warn(clippy::all, rust_2018_idioms)]

mod gamut_data;

use godot::prelude::*;

struct GamutPlottyExt;

#[gdextension]
unsafe impl ExtensionLibrary for GamutPlottyExt {}
