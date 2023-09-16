/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

pub mod mapfile;
pub mod segment;
pub mod file;
pub mod symbol;
pub mod utils;

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
        let map = MapFile::new();
        map.read_map_file(&"tests/maps/w0_000.map".to_string());
    }
}
