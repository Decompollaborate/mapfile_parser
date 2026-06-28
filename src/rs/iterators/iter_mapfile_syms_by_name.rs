/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::iter::FusedIterator;

use super::IterSegmentSymsByName;
use crate::{MapFile, MaybeFoundSymbolInfo};

#[derive(Debug, Clone)]
pub struct IterMapFileSymsByName<'map, 'name> {
    mapfile: &'map MapFile,
    index: usize,
    sym_name: &'name str,
    iter: Option<IterSegmentSymsByName<'map, 'name>>,
}

impl<'map, 'name> IterMapFileSymsByName<'map, 'name> {
    pub(crate) fn new(mapfile: &'map MapFile, sym_name: &'name str) -> Self {
        Self {
            mapfile,
            index: 0,
            sym_name,
            iter: None,
        }
    }

    fn next_impl(&mut self) -> Option<MaybeFoundSymbolInfo<'map>> {
        while self.index < self.mapfile.segments_list.len() {
            if let Some(iter) = &mut self.iter {
                if let Some(next) = iter.next() {
                    return Some(next);
                }
                self.iter = None;
            }

            let segment = &self.mapfile.segments_list[self.index];
            self.index += 1;

            self.iter = Some(segment.find_possible_symbols_by_name(self.sym_name));
        }

        None
    }
}

impl<'map, 'name> Iterator for IterMapFileSymsByName<'map, 'name> {
    type Item = MaybeFoundSymbolInfo<'map>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.mapfile.segments_list.len() - self.index))
    }
}

impl<'map, 'name> FusedIterator for IterMapFileSymsByName<'map, 'name> {}
