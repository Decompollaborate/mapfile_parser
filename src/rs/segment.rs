/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{file, found_symbol_info};
use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[pyclass(module = "mapfile_parser", unsendable)]
pub struct Segment {
    #[pyo3(get, set)]
    pub name: String,

    #[pyo3(get, set)]
    pub vram: u64,

    #[pyo3(get, set)]
    pub size: u64,

    #[pyo3(get, set)]
    pub vrom: u64,

    // #[pyo3(get, set)]
    pub files_list: Vec<file::File>,
}

#[pymethods]
impl Segment {
    #[new]
    pub fn new(name: String, vram: u64, size: u64, vrom: u64) -> Self {
        Segment {
            name: name.into(),
            vram: vram,
            size: size,
            vrom: vrom,
            files_list: Vec::new(),
        }
    }

    pub fn find_symbol_by_name(&self, sym_name: &str) -> Option<found_symbol_info::FoundSymbolInfo> {
        for file in &self.files_list {
            if let Some(sym) = file.find_symbol_by_name(sym_name) {
                return Some(found_symbol_info::FoundSymbolInfo::new_default(file.clone(), sym));
            }
        }
        None
    }
}
