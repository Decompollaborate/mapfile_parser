/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{utils, file};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub vram: u64,
    pub size: Option<u64>,
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
