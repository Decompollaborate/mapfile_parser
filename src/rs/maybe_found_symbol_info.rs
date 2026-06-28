/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::borrow::Cow;

use crate::{found_symbol_info, section, segment, symbol};

#[derive(Debug, Clone)]
pub struct MaybeFoundSymbolInfo<'a> {
    segment: &'a segment::Segment,
    section: &'a section::Section,
    symbol: Option<&'a symbol::Symbol>,
    offset: i64,
}

impl<'a> MaybeFoundSymbolInfo<'a> {
    pub(crate) fn new(
        segment: &'a segment::Segment,
        section: &'a section::Section,
        symbol: Option<&'a symbol::Symbol>,
        offset: i64,
    ) -> Self {
        Self {
            segment,
            section,
            symbol,
            offset,
        }
    }

    pub fn segment(&self) -> &'a segment::Segment {
        self.segment
    }
    pub fn section(&self) -> &'a section::Section {
        self.section
    }
    pub fn symbol(&self) -> Option<&'a symbol::Symbol> {
        self.symbol
    }
    pub fn offset(&self) -> i64 {
        self.offset
    }

    pub fn get_as_str_plus_offset(&self, sym_name: &str) -> String {
        if let Some(symbol) = self.symbol {
            let info = found_symbol_info::FoundSymbolInfo::new(self.section, symbol, self.offset);
            let extra = Cow::from(format!(", SEG: {}", self.segment.name));

            info.get_as_str_plus_offset_impl(Some(Cow::from(sym_name)), extra)
        } else {
            let extra = if self.offset != 0 {
                Cow::from(format!(" at offset 0x{:X}", self.offset))
            } else {
                Cow::from("")
            };

            format!(
                "{} may be part of section {} (segment {}){}, but it isn't globally visible.",
                sym_name,
                self.section.filepath.to_string_lossy(),
                self.segment.name,
                extra,
            )
        }
    }
}
