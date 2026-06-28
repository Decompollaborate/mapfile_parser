/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::iter::FusedIterator;

use crate::{MapFile, MaybeFoundSymbolInfo};

#[derive(Debug, Clone)]
pub struct IterMapFileSymsByVram<'map> {
    mapfile: &'map MapFile,
    index: usize,
    address: u64,
}

impl<'map> IterMapFileSymsByVram<'map> {
    pub(crate) fn new(mapfile: &'map MapFile, address: u64) -> Self {
        Self {
            mapfile,
            index: 0,
            address,
        }
    }

    fn next_impl(&mut self) -> Option<MaybeFoundSymbolInfo<'map>> {
        while self.index < self.mapfile.segments_list.len() {
            let segment = &self.mapfile.segments_list[self.index];
            self.index += 1;

            if let Some(sym) = segment.find_possible_symbol_by_vram(self.address) {
                return Some(sym);
            }
        }

        None
    }
}

impl<'map> Iterator for IterMapFileSymsByVram<'map> {
    type Item = MaybeFoundSymbolInfo<'map>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.mapfile.segments_list.len() - self.index))
    }
}

impl<'map> FusedIterator for IterMapFileSymsByVram<'map> {}
