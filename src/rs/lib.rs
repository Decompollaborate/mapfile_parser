/* SPDX-FileCopyrightText: © 2023-2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod file;
mod found_symbol_info;
mod mapfile;
mod maps_comparison_info;
mod progress_stats;
mod segment;
mod symbol;
mod symbol_comparison_info;
pub mod utils;

pub use file::File;
pub use found_symbol_info::FoundSymbolInfo;
pub use mapfile::MapFile;
pub use maps_comparison_info::MapsComparisonInfo;
pub use progress_stats::ProgressStats;
pub use segment::Segment;
pub use symbol::Symbol;
pub use symbol_comparison_info::SymbolComparisonInfo;

#[macro_use]
extern crate lazy_static;

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python_bindings")]
#[pymodule]
fn mapfile_parser(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<mapfile::MapFile>()?;
    m.add_class::<segment::Segment>()?;
    m.add_class::<file::File>()?;
    m.add_class::<symbol::Symbol>()?;
    m.add_class::<found_symbol_info::FoundSymbolInfo>()?;
    m.add_class::<symbol_comparison_info::SymbolComparisonInfo>()?;
    m.add_class::<maps_comparison_info::MapsComparisonInfo>()?;
    m.add_class::<progress_stats::ProgressStats>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::mapfile::MapFile;

    // TODO: tests

    #[test]
    fn w0_000_map() {
        let mut map = MapFile::new();
        map.read_map_file(&PathBuf::from("tests/maps/w0_000.map"));
    }
}
