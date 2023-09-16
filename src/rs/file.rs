/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{utils, symbol};

#[derive(Debug, Clone)]
#[pyo3::prelude::pyclass(module = "mapfile_parser", unsendable)]
pub struct File {
    #[pyo3(get, set)]
    pub filepath: std::path::PathBuf,

    #[pyo3(get, set)]
    pub vram: u64,

    #[pyo3(get, set)]
    pub size: u64,

    #[pyo3(get, set)]
    pub section_type: String,

    #[pyo3(get, set)]
    pub vrom: Option<u64>,

    //#[pyo3(get, set)]
    pub symbols: Vec<symbol::Symbol>,
}

#[pyo3::prelude::pymethods]
impl File {
    #[new]
    pub fn new(filepath: std::path::PathBuf, vram: u64, size: u64, section_type: &str) -> Self {
        File {
            filepath: filepath,
            vram: vram,
            size: size,
            section_type: section_type.into(),
            vrom: None,
            symbols: Vec::new(),
        }
    }

    pub fn is_noload_section(&self) -> bool {
        return self.section_type == ".bss";
    }
}
