/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::path::PathBuf;

use crate::{section, symbol};

pub enum SymbolDecompState<'sect> {
    Decomped(&'sect symbol::Symbol),
    Undecomped(&'sect symbol::Symbol),
}

pub struct SymbolDecompStateIter<'sect> {
    section: &'sect section::Section,
    whole_file_is_undecomped: bool,
    functions_path: Option<PathBuf>,

    index: usize,
}

impl<'sect> SymbolDecompStateIter<'sect> {
    pub(crate) fn new(
        section: &'sect section::Section,
        whole_file_is_undecomped: bool,
        functions_path: Option<PathBuf>,
    ) -> Self {
        Self {
            section,
            whole_file_is_undecomped,
            functions_path,

            index: 0,
        }
    }
}

impl<'sect> Iterator for SymbolDecompStateIter<'sect> {
    type Item = SymbolDecompState<'sect>;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip over `.NON_MATCHING` symbols
        while self.index < self.section.symbols.len() {
            let sym = &self.section.symbols[self.index];
            if !sym.name.ends_with(".NON_MATCHING") {
                break;
            }
            self.index += 1;
        }
        if self.index >= self.section.symbols.len() {
            return None;
        }

        let sym = &self.section.symbols[self.index];
        self.index += 1;

        if self.whole_file_is_undecomped || sym.nonmatching_sym_exists {
            return Some(SymbolDecompState::Undecomped(sym));
        } else if let Some(functions_path) = &self.functions_path {
            if functions_path.join(sym.name.clone() + ".s").exists() {
                return Some(SymbolDecompState::Undecomped(sym));
            }
        }

        Some(SymbolDecompState::Decomped(sym))
    }
}
