/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{utils, file};

#[derive(Debug, Clone)]
pub struct Segment {
    pub name: String,
    pub vram: u64,
    pub size: u64,
    pub vrom: u64,
    files_list: Vec<file::File>,
}

impl Segment {
    pub fn new(name: &String, vram: u64, size: u64, vrom: u64) -> Self {
        Segment {
            name: name.into(),
            vram: vram,
            size: size,
            vrom: vrom,
            files_list: Vec::new(),
        }
    }
}
