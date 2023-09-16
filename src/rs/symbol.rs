/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{utils, file};

#[derive(Debug, Clone)]
#[pyo3::prelude::pyclass(module = "mapfile_parser", unsendable)]
pub struct Symbol {
    #[pyo3(get, set)]
    pub name: String,

    #[pyo3(get, set)]
    pub vram: u64,

    #[pyo3(get, set)]
    pub size: Option<u64>,

    #[pyo3(get, set)]
    pub vrom: Option<u64>,
}

impl Symbol {
    pub fn new(name: &String, vram: u64) -> Self {
        Symbol {
            name: name.into(),
            vram: vram,
            size: None,
            vrom: None,
        }
    }
}
