//! Core library modules: weather logic, formatting, colors, and constants.
//! These are independent of CLI (`cli`) and terminal rendering (`terminal`).

pub mod cache;
pub mod color;
pub mod constants;
pub mod error;
pub mod format;
pub mod graphs;
pub mod l10n;
pub mod location;
pub mod location_remote;
pub mod warnings;
pub mod weather;
