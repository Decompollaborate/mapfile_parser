/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod mapfile;
mod segment;
mod file;
mod symbol;
mod found_symbol_info;
mod json_element;
pub mod utils;

pub use mapfile::MapFile;
pub use segment::Segment;
pub use file::File;
pub use symbol::Symbol;
pub use found_symbol_info::FoundSymbolInfo;
pub use json_element::JsonElement;

#[pyo3::prelude::pymodule]
fn mapfile_parser(_py: pyo3::prelude::Python<'_>, m: &pyo3::prelude::PyModule) -> pyo3::prelude::PyResult<()> {
    m.add_class::<mapfile::MapFile>()?;
    m.add_class::<segment::Segment>()?;
    m.add_class::<file::File>()?;
    m.add_class::<symbol::Symbol>()?;
    m.add_class::<found_symbol_info::FoundSymbolInfo>()?;
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
        map.read_map_file("tests/maps/w0_000.map");
    }
}