/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::path::PathBuf;

use crate::{file, symbol};

pub enum SymbolDecompState<'sect> {
    Decomped(&'sect symbol::Symbol),
    Undecomped(&'sect symbol::Symbol),
}

pub struct SymbolDecompStateIter<'sect> {
    section: &'sect file::File,
    whole_file_is_undecomped: bool,
    check_function_files: bool,
    functions_path: Option<PathBuf>,

    index: usize,
}

impl<'sect> SymbolDecompStateIter<'sect> {
    pub(crate) fn new(
        section: &'sect file::File,
        whole_file_is_undecomped: bool,
        check_function_files: bool,
        functions_path: Option<PathBuf>,
    ) -> Self {
        Self {
            section,
            whole_file_is_undecomped,
            check_function_files,
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

        if self.whole_file_is_undecomped
            || self
                .section
                .find_symbol_by_name(&format!("{}.NON_MATCHING", sym.name))
                .is_some()
        {
            return Some(SymbolDecompState::Undecomped(sym));
        } else if self.check_function_files {
            if let Some(functions_path) = &self.functions_path {
                if functions_path.join(sym.name.clone() + ".s").exists() {
                    return Some(SymbolDecompState::Undecomped(sym));
                }
            }
        }

        Some(SymbolDecompState::Decomped(sym))
    }
}
