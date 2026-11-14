//! # dev-scrub
//!
//! Surgical Developer Disk Hygiene, Artifact Cleaner & Git-Aware Cache Vacuum in Rust.

pub mod cleaner;
pub mod scanner;
pub mod ui;

pub use cleaner::{CleanOptions, CleanResult, VacuumEngine};
pub use scanner::{ArtifactItem, ArtifactType, DevScanner, GitActivity, GitInspector, ScanReport};
pub use ui::TerminalHud;
