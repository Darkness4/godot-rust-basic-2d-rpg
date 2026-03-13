pub mod death;
pub mod enemy;
pub mod player;
pub(crate) mod traits;

use godot::prelude::*;

struct Basic2dRpg;

#[gdextension]
unsafe impl ExtensionLibrary for Basic2dRpg {}
