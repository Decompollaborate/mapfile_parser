/* SPDX-FileCopyrightText: © 2023-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#![warn(clippy::manual_let_else)]

mod found_symbol_info;
mod mapfile;
mod maps_comparison_info;
mod parser;
mod progress_stats;
mod section;
mod segment;
mod symbol;
mod symbol_comparison_info;
mod symbol_decomp_state;
pub mod utils;

#[cfg(feature = "objdiff_report")]
pub mod report;

pub use found_symbol_info::FoundSymbolInfo;
pub use mapfile::MapFile;
pub use maps_comparison_info::MapsComparisonInfo;
pub use progress_stats::ProgressStats;
pub use section::{PathDecompSettings, Section};
pub use segment::Segment;
pub use symbol::Symbol;
pub use symbol_comparison_info::SymbolComparisonInfo;
pub use symbol_decomp_state::{SymbolDecompState, SymbolDecompStateIter};

// Renamed types
#[deprecated(since = "2.8.0", note = "Use `Section` instead")]
pub use Section as File;

#[macro_use]
extern crate lazy_static;

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python_bindings")]
#[pymodule]
fn mapfile_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<mapfile::MapFile>()?;
    m.add_class::<segment::Segment>()?;
    m.add_class::<section::Section>()?;
    m.add_class::<symbol::Symbol>()?;
    m.add_class::<found_symbol_info::python_bindings::PyFoundSymbolInfo>()?;
    m.add_class::<symbol_comparison_info::python_bindings::PySymbolComparisonInfo>()?;
    m.add_class::<maps_comparison_info::python_bindings::PyMapsComparisonInfo>()?;
    m.add_class::<progress_stats::ProgressStats>()?;
    m.add_class::<report::ReportCategories>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::mapfile::MapFile;

    // TODO: tests

    #[test]
    fn w0_000_map() {
        let _ = MapFile::new_from_map_file(&PathBuf::from("tests/maps/w0_000.map"));
    }
}
