//! Game domains — each subdirectory is worker-owned.
//! Workers import from crate::shared, never redefine types locally.
#[allow(unused_imports)]
use crate::shared;

pub mod combat;
pub mod damage_mods;
pub mod data_loader;
pub mod status;
