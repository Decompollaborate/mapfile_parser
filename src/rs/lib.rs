/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
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

#[pyo3::prelude::pymodule]
fn mapfile_parser(
    _py: pyo3::prelude::Python<'_>,
    m: &pyo3::prelude::PyModule,
) -> pyo3::prelude::PyResult<()> {
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

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use crate::mapfile::MapFile;

    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn w0_000_map() {
        let mut map = MapFile::new();
        map.read_map_file("tests/maps/w0_000.map".into());
    }
}
