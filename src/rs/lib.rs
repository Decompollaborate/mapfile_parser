/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod mapfile;
mod segment;
mod file;
mod symbol;
mod found_symbol_info;
pub mod utils;

pub use mapfile::MapFile;
pub use segment::Segment;
pub use file::File;
pub use symbol::Symbol;
pub use found_symbol_info::FoundSymbolInfo;

#[pyo3::prelude::pymodule]
fn mapfile_parser(_py: pyo3::prelude::Python<'_>, m: &pyo3::prelude::PyModule) -> pyo3::prelude::PyResult<()> {
    m.add_class::<mapfile::MapFile>()?;
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
