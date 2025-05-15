/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::path::PathBuf;

use crate::{section, symbol};

pub enum SymbolDecompState<'sect> {
    Decomped(&'sect symbol::Symbol),
    // Returns a new symbol because it may need the size patched
    Undecomped(symbol::Symbol),
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

        // TODO: move `.NON_MATCHING` handling to the parsing itself instead.
        if let Some(non_matching_sym) = self
            .section
            .find_symbol_by_name(&format!("{}.NON_MATCHING", sym.name))
        {
            let mut undecomped_sym = sym.clone();
            if undecomped_sym.size == 0 {
                undecomped_sym.size = non_matching_sym.size;
            }
            return Some(SymbolDecompState::Undecomped(undecomped_sym));
        }

        if self.whole_file_is_undecomped {
            return Some(SymbolDecompState::Undecomped(sym.clone()));
        } else if let Some(functions_path) = &self.functions_path {
            if functions_path.join(sym.name.clone() + ".s").exists() {
                return Some(SymbolDecompState::Undecomped(sym.clone()));
            }
        }

        Some(SymbolDecompState::Decomped(sym))
    }
}
