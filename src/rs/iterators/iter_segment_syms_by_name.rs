/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::iter::FusedIterator;

use crate::{MaybeFoundSymbolInfo, Segment};

#[derive(Debug, Clone)]
pub struct IterSegmentSymsByName<'seg, 'name> {
    segment: &'seg Segment,
    index: usize,
    sym_name: &'name str,
}

impl<'seg, 'name> IterSegmentSymsByName<'seg, 'name> {
    pub(crate) fn new(segment: &'seg Segment, sym_name: &'name str) -> Self {
        Self {
            segment,
            index: 0,
            sym_name,
        }
    }

    fn next_impl(&mut self) -> Option<MaybeFoundSymbolInfo<'seg>> {
        while self.index < self.segment.sections_list.len() {
            let section = &self.segment.sections_list[self.index];
            self.index += 1;

            if let Some(sym) = section.find_symbol_by_name(self.sym_name) {
                return Some(MaybeFoundSymbolInfo::new(
                    self.segment,
                    section,
                    Some(sym),
                    0,
                ));
            }
        }

        None
    }
}

impl<'seg, 'name> Iterator for IterSegmentSymsByName<'seg, 'name> {
    type Item = MaybeFoundSymbolInfo<'seg>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.segment.sections_list.len() - self.index))
    }
}

impl<'seg, 'name> FusedIterator for IterSegmentSymsByName<'seg, 'name> {}
