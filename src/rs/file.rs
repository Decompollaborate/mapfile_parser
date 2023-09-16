/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{utils, symbol};

#[derive(Debug, Clone)]
pub struct File {
    pub filepath: std::path::PathBuf,
    pub vram: u64,
    pub size: u64,
    pub section_type: String,
    pub vrom: Option<u64>,
    pub symbols: Vec<symbol::Symbol>,
}

impl File {
    pub fn new(filepath: &std::path::PathBuf, vram: u64, size: u64, section_type: &String) -> Self {
        File {
            filepath: filepath.into(),
            vram: vram,
            size: size,
            section_type: section_type.into(),
            vrom: None,
            symbols: Vec::new(),
        }
    }
}
