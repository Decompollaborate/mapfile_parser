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

#[pyo3::prelude::pymethods]
impl Symbol {
    #[new]
    pub fn new(name: String, vram: u64) -> Self {
        Symbol {
            name: name.into(),
            vram: vram,
            size: None,
            vrom: None,
        }
    }

    #[pyo3(name = "getVramStr")]
    pub fn get_vram_str(&self) -> String {
        format!("0x{0:08X}", self.vram)
    }

    #[pyo3(name = "getSizeStr")]
    pub fn get_size_str(&self) -> String {
        if let Some(size) = self.size {
            return format!("0x{0:X}", size);
        }
        "None".into()
    }

    #[pyo3(name = "getVromStr")]
    pub fn get_vrom_str(&self) -> String {
        if let Some(vrom) = self.vrom {
            return format!("0x{0:06X}", vrom);
        }
        "None".into()
    }
}
