/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{utils, file};

#[derive(Debug, Clone)]
#[pyo3::prelude::pyclass(module = "mapfile_parser", unsendable)]
pub struct Segment {
    #[pyo3(get, set)]
    pub name: String,

    #[pyo3(get, set)]
    pub vram: u64,

    #[pyo3(get, set)]
    pub size: u64,

    #[pyo3(get, set)]
    pub vrom: u64,

    #[pyo3(get, set)]
    pub files_list: Vec<file::File>,
}

#[pyo3::prelude::pymethods]
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
}
