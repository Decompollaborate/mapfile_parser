/* SPDX-FileCopyrightText: © 2023-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    path::Path,
};

use regex::*;

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    found_symbol_info, maps_comparison_info, progress_stats, section, segment, symbol,
    symbol_comparison_info, symbol_decomp_state, utils,
};

lazy_static! {
    static ref BANNED_SYMBOL_NAMES: HashSet<&'static str> = {
        let mut symbol_names = HashSet::new();
        symbol_names.insert("gcc2_compiled.");
        symbol_names
    };
}

#[derive(Debug, Clone)]
// TODO: sequence?
#[cfg_attr(feature = "python_bindings", pyclass(module = "mapfile_parser"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MapFile {
    pub segments_list: Vec<segment::Segment>,

    #[cfg(feature = "python_bindings")]
    #[pyo3(get, set)]
    debugging: bool,
}

impl MapFile {
    pub fn new() -> Self {
        Self {
            segments_list: Vec::new(),

            #[cfg(feature = "python_bindings")]
            debugging: false,
        }
    }

    /// Creates a new `MapFile` object and fills it with the contents from the
    /// file pointed by the `map_path` argument.
    ///
    /// The format of the map will be guessed based on its contents.
    ///
    /// Currently supported map formats:
    /// - GNU ld
    /// - clang ld.lld
    pub fn new_from_map_file(map_path: &Path) -> Self {
        let mut m = Self::new();
        m.read_map_file(map_path);
        m
    }

    /**
    Opens the mapfile pointed by the `map_path` argument and parses it.

    The format of the map will be guessed based on its contents.

    Currently supported map formats:
    - GNU ld
    - clang ld.lld
     */
    pub fn read_map_file(&mut self, map_path: &Path) {
        let map_contents = utils::read_file_contents(map_path);

        self.parse_map_contents(&map_contents);
    }

    /**
    Parses the contents of the map.

    The `map_contents` argument must contain the contents of a mapfile.

    The format of the map will be guessed based on its contents.

    Currently supported mapfile formats:
    - GNU ld
    - clang ld.lld
    */
    pub fn parse_map_contents(&mut self, map_contents: &str) {
        let regex_lld_header =
            Regex::new(r"\s+VMA\s+LMA\s+Size\s+Align\s+Out\s+In\s+Symbol").unwrap();

        if regex_lld_header.is_match(map_contents) {
            self.parse_map_contents_lld(map_contents);
        } else {
            // GNU is the fallback
            self.parse_map_contents_gnu(map_contents);
        }
    }

    /**
    Parses the contents of a GNU ld map.

    The `map_contents` argument must contain the contents of a GNU ld mapfile.
     */
    pub fn parse_map_contents_gnu(&mut self, map_contents: &str) {
        // TODO: maybe move somewhere else?
        let regex_section_alone_entry = Regex::new(r"^\s+(?P<section>[^*][^\s]+)\s*$").unwrap();
        let regex_section_data_entry = Regex::new(r"^\s+(?P<section>([^*][^\s]+)?)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s+(?P<name>[^\s]+)$").unwrap();
        let regex_function_entry =
            Regex::new(r"^\s+(?P<vram>0x[^\s]+)\s+(?P<name>[^\s]+)$").unwrap();
        // regex_function_entry = re.compile(r"^\s+(?P<vram>0x[^\s]+)\s+(?P<name>[^\s]+)((\s*=\s*(?P<expression>.+))?)$")
        let regex_label = Regex::new(r"(?P<name>\.?L[0-9A-F]{8})$").unwrap();
        let regex_fill =
            Regex::new(r"^\s+(?P<fill>\*[^\s\*]+\*)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s+(?P<fillValue>[0-9a-zA-Z]*)$")
                .unwrap();
        let regex_segment_entry = Regex::new(r"(?P<name>([^\s]+)?)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s+(?P<loadaddress>(load address)?)\s+(?P<vrom>0x[^\s]+)$").unwrap();

        let map_data = MapFile::preprocess_map_data_gnu(map_contents);

        let mut temp_segment_list = vec![segment::Segment::new_placeholder()];

        let mut in_section = false;

        let mut prev_line = "";
        for line in map_data.split('\n') {
            if in_section {
                if !line.starts_with("        ") {
                    in_section = false;
                } else if !regex_label.is_match(line) {
                    // Filter out jump table's labels

                    // Find symbols
                    if let Some(entry_match) = regex_function_entry.captures(line) {
                        // println!("{entry_match:?}");
                        let sym_name = &entry_match["name"];

                        if !BANNED_SYMBOL_NAMES.contains(&sym_name) {
                            let sym_vram = utils::parse_hex(&entry_match["vram"]);

                            let current_segment = temp_segment_list.last_mut().unwrap();
                            let current_section = current_segment.sections_list.last_mut().unwrap();

                            current_section
                                .symbols
                                .push(symbol::Symbol::new_default(sym_name.into(), sym_vram));
                        }
                    }
                }
            }

            if !in_section {
                if let Some(section_entry_match) = regex_section_data_entry.captures(line) {
                    let filepath = std::path::PathBuf::from(&section_entry_match["name"]);
                    let vram = utils::parse_hex(&section_entry_match["vram"]);
                    let size = utils::parse_hex(&section_entry_match["size"]);
                    let section_type = &section_entry_match["section"];

                    if size > 0 {
                        // TODO: de-duplicate the following code:

                        if !section_type.is_empty() {
                            in_section = true;
                            let current_segment = temp_segment_list.last_mut().unwrap();

                            current_segment
                                .sections_list
                                .push(section::Section::new_default(
                                    filepath,
                                    vram,
                                    size,
                                    section_type,
                                ));
                        } else if let Some(section_alone_match) =
                            regex_section_alone_entry.captures(prev_line)
                        {
                            // Some sections may be too large, making the entry be splitted between two lines, making the section name be in one line and the rest of the info in the next one

                            let section_type = &section_alone_match["section"];

                            in_section = true;
                            let current_segment = temp_segment_list.last_mut().unwrap();

                            current_segment
                                .sections_list
                                .push(section::Section::new_default(
                                    filepath,
                                    vram,
                                    size,
                                    section_type,
                                ));
                        }
                    }
                } else if let Some(segment_entry_match) = regex_segment_entry.captures(line) {
                    let mut name = &segment_entry_match["name"];
                    let vram = utils::parse_hex(&segment_entry_match["vram"]);
                    let size = utils::parse_hex(&segment_entry_match["size"]);
                    let vrom = utils::parse_hex(&segment_entry_match["vrom"]);

                    if name.is_empty() {
                        // If the segment name is too long then this line gets break in two lines
                        name = prev_line;
                    }

                    temp_segment_list.push(segment::Segment::new_default(
                        name.into(),
                        vram,
                        size,
                        vrom,
                    ));
                } else if let Some(fill_match) = regex_fill.captures(line) {
                    // Make a dummy file to handle *fill*
                    let mut filepath = std::path::PathBuf::new();
                    let mut vram = 0;
                    let size = utils::parse_hex(&fill_match["size"]);
                    let mut section_type = "".to_owned();

                    let current_segment = temp_segment_list.last_mut().unwrap();

                    if !current_segment.sections_list.is_empty() {
                        let prev_section = current_segment.sections_list.last().unwrap();
                        let mut name = prev_section.filepath.file_name().unwrap().to_owned();

                        name.push("__fill__");
                        filepath = prev_section.filepath.with_file_name(name);
                        vram = prev_section.vram + prev_section.size;
                        section_type.clone_from(&prev_section.section_type);
                    }

                    current_segment
                        .sections_list
                        .push(section::Section::new_default(
                            filepath,
                            vram,
                            size,
                            &section_type,
                        ));
                }
            }

            prev_line = line;
        }

        self.segments_list = Self::post_process_segments_gnu(temp_segment_list);
    }

    fn post_process_segments_gnu(
        temp_segment_list: Vec<segment::Segment>,
    ) -> Vec<segment::Segment> {
        let mut segments_list = Vec::with_capacity(temp_segment_list.len());

        for (i, segment) in temp_segment_list.into_iter().enumerate() {
            if i == 0 && segment.is_placeholder() {
                // skip the dummy segment if it has no size, sections or symbols
                continue;
            }

            let mut new_segment = segment.clone_no_sectionlist();

            let mut vrom_offset = segment.vrom;
            for mut section in segment.sections_list.into_iter() {
                let mut acummulated_size = 0;
                let symbols_count = section.symbols.len();
                let is_noload_section = section.is_noload_section();

                if section.is_placeholder() {
                    // drop placeholders
                    continue;
                }

                if section.vrom.is_some() {
                    vrom_offset = section.vrom.unwrap();
                }

                if !is_noload_section {
                    section.vrom = Some(vrom_offset);
                }

                if symbols_count > 0 {
                    let mut sym_vrom = vrom_offset;

                    // The first symbol of the section on the mapfile may not be the actual first
                    // symbol if it is marked `static`, be a jumptable, etc, producing a mismatch
                    // on the vrom address of each symbol of this section.
                    // A way to adjust this difference is by increasing the start of the vrom
                    // by the difference in vram address between the first symbol and the vram
                    // of the section.
                    if let Some(first_sym) = section.symbols.first() {
                        sym_vrom += first_sym.vram - section.vram;

                        // Aditionally, if the first symbol is missing then calculation of the size
                        // for the last symbol would be wrong, since we subtract the accumulated
                        // size of each symbol from the section's total size to calculate it.
                        // We need to adjust the total size by this difference too.
                        acummulated_size += first_sym.vram - section.vram;
                    }

                    // Calculate size of each symbol
                    for index in 0..symbols_count - 1 {
                        let next_sym_vram = section.symbols[index + 1].vram;
                        let sym = &mut section.symbols[index];
                        let sym_size = next_sym_vram - sym.vram;
                        acummulated_size += sym_size;

                        sym.size = sym_size;

                        if !is_noload_section {
                            // Only set vrom of non bss variables
                            sym.vrom = Some(sym_vrom);
                            sym_vrom += sym_size;
                        }
                    }

                    // Calculate size of last symbol of the section
                    let sym = &mut section.symbols[symbols_count - 1];
                    let sym_size = section.size - acummulated_size;
                    sym.size = sym_size;
                    if !is_noload_section {
                        sym.vrom = Some(sym_vrom);
                        //sym_vrom += sym_size;
                    }
                }

                if !is_noload_section {
                    vrom_offset += section.size;
                }

                new_segment.sections_list.push(section);
            }

            segments_list.push(new_segment);
        }

        segments_list.shrink_to_fit();
        segments_list
    }

    /**
    Parses the contents of a clang ld.lld map.

    The `map_contents` argument must contain the contents of a clang ld.lld mapfile.
     */
    pub fn parse_map_contents_lld(&mut self, map_contents: &str) {
        let map_data = map_contents;

        // Every line starts with this information, so instead of duplicating it we put them on one single regex
        let regex_row_entry = Regex::new(r"^\s*(?P<vram>[0-9a-fA-F]+)\s+(?P<vrom>[0-9a-fA-F]+)\s+(?P<size>[0-9a-fA-F]+)\s+(?P<align>[0-9a-fA-F]+) ").unwrap();

        let regex_segment_entry = Regex::new(r"^(?P<name>[^\s]+)$").unwrap();
        let regex_fill = Regex::new(r"^\s+(?P<expr>\.\s*\+=\s*.+)$").unwrap();
        let regex_section_data_entry =
            Regex::new(r"^\s+(?P<name>[^\s]+):\((?P<section>[^\s()]+)\)$$").unwrap();
        let regex_label = Regex::new(r"^\s+(?P<name>\.?L[0-9A-F]{8})$").unwrap();
        let regex_symbol_entry = Regex::new(r"^\s+(?P<name>[^\s]+)$").unwrap();

        let mut temp_segment_list = vec![segment::Segment::new_placeholder()];

        for line in map_data.split('\n') {
            if let Some(row_entry_match) = regex_row_entry.captures(line) {
                let vram = utils::parse_hex(&row_entry_match["vram"]);
                let vrom = utils::parse_hex(&row_entry_match["vrom"]);
                let size = utils::parse_hex(&row_entry_match["size"]);
                let align = utils::parse_hex(&row_entry_match["align"]);

                let subline = &line[row_entry_match.get(0).unwrap().len()..];

                if let Some(segment_entry_match) = regex_segment_entry.captures(subline) {
                    let name = &segment_entry_match["name"];

                    let mut new_segment =
                        segment::Segment::new_default(name.into(), vram, size, vrom);
                    new_segment.align = Some(align);

                    temp_segment_list.push(new_segment);
                } else if regex_fill.is_match(subline) {
                    // Make a dummy section to handle pads (. += XX)

                    let mut filepath = std::path::PathBuf::new();
                    let mut section_type = "".to_owned();

                    let current_segment = temp_segment_list.last_mut().unwrap();

                    if !current_segment.sections_list.is_empty() {
                        let prev_section = current_segment.sections_list.last().unwrap();
                        let mut name = prev_section.filepath.file_name().unwrap().to_owned();

                        name.push("__fill__");
                        filepath = prev_section.filepath.with_file_name(name);
                        section_type.clone_from(&prev_section.section_type);
                    }

                    let mut new_section =
                        section::Section::new_default(filepath, vram, size, &section_type);
                    if !utils::is_noload_section(&section_type) {
                        new_section.vrom = Some(vrom);
                    }
                    current_segment.sections_list.push(new_section);
                } else if let Some(section_entry_match) = regex_section_data_entry.captures(subline)
                {
                    let filepath = std::path::PathBuf::from(&section_entry_match["name"]);
                    let section_type = &section_entry_match["section"];

                    if size > 0 {
                        let current_segment = temp_segment_list.last_mut().unwrap();

                        let mut new_section =
                            section::Section::new_default(filepath, vram, size, section_type);
                        if !utils::is_noload_section(section_type) {
                            new_section.vrom = Some(vrom);
                        }
                        new_section.align = Some(align);

                        current_segment.sections_list.push(new_section);
                    }
                } else if regex_label.is_match(subline) {
                    // pass
                } else if let Some(symbol_entry_match) = regex_symbol_entry.captures(subline) {
                    let name = &symbol_entry_match["name"];

                    if !BANNED_SYMBOL_NAMES.contains(&name) {
                        let current_segment = temp_segment_list.last_mut().unwrap();
                        let current_section = current_segment.sections_list.last_mut().unwrap();

                        let mut new_symbol = symbol::Symbol::new_default(name.into(), vram);
                        if size > 0 {
                            new_symbol.size = size;
                        }
                        if !current_section.is_noload_section() {
                            new_symbol.vrom = Some(vrom)
                        }
                        new_symbol.align = Some(align);

                        current_section.symbols.push(new_symbol);
                    }
                }
            }
        }

        self.segments_list = Self::post_process_segments_lld(temp_segment_list);
    }

    fn post_process_segments_lld(
        temp_segment_list: Vec<segment::Segment>,
    ) -> Vec<segment::Segment> {
        let mut segments_list = Vec::with_capacity(temp_segment_list.len());

        for (i, segment) in temp_segment_list.into_iter().enumerate() {
            if i == 0 && segment.is_placeholder() {
                // skip the dummy segment if it has no size, sections or symbols
                continue;
            }

            let mut new_segment = segment.clone_no_sectionlist();

            for mut section in segment.sections_list.into_iter() {
                if section.is_placeholder() {
                    // drop placeholders
                    continue;
                }

                let mut acummulated_size = 0;
                let symbols_count = section.symbols.len();

                if symbols_count > 0 {
                    // Calculate the size of symbols that the map section did not report.
                    // usually asm symbols and not C ones

                    for index in 0..symbols_count - 1 {
                        let next_sym_vram = section.symbols[index + 1].vram;
                        let sym = &mut section.symbols[index];

                        let sym_size = next_sym_vram - sym.vram;
                        acummulated_size += sym_size;

                        if sym.size == 0 {
                            sym.size = sym_size;
                        }
                    }

                    // Calculate size of last symbol of the section
                    let sym = &mut section.symbols[symbols_count - 1];
                    if sym.size == 0 {
                        let sym_size = section.size - acummulated_size;
                        sym.size = sym_size;
                    }
                }

                new_segment.sections_list.push(section);
            }

            segments_list.push(new_segment);
        }

        segments_list.shrink_to_fit();
        segments_list
    }

    pub fn filter_by_section_type(&self, section_type: &str) -> Self {
        let mut new_map_file = MapFile::new();

        for segment in &self.segments_list {
            let new_segment = segment.filter_by_section_type(section_type);

            if !new_segment.sections_list.is_empty() {
                new_map_file.segments_list.push(new_segment);
            }
        }

        new_map_file
    }

    pub fn get_every_section_except_section_type(&self, section_type: &str) -> Self {
        let mut new_map_file = MapFile::new();

        for segment in &self.segments_list {
            let new_segment = segment.get_every_section_except_section_type(section_type);

            if !new_segment.sections_list.is_empty() {
                new_map_file.segments_list.push(new_segment);
            }
        }

        new_map_file
    }

    #[deprecated(
        since = "2.8.0",
        note = "Use `get_every_section_except_section_type` instead"
    )]
    pub fn get_every_file_except_section_type(&self, section_type: &str) -> Self {
        self.get_every_section_except_section_type(section_type)
    }

    pub fn find_symbol_by_name(
        &self,
        sym_name: &str,
    ) -> Option<found_symbol_info::FoundSymbolInfo> {
        for segment in &self.segments_list {
            if let Some(info) = segment.find_symbol_by_name(sym_name) {
                return Some(info);
            }
        }

        None
    }

    #[deprecated(
        since = "2.7.0",
        note = "Use `find_symbol_by_vram` or `find_symbol_by_vrom` instead."
    )]
    pub fn find_symbol_by_vram_or_vrom(
        &self,
        address: u64,
    ) -> Option<found_symbol_info::FoundSymbolInfo> {
        for segment in &self.segments_list {
            #[allow(deprecated)]
            if let Some(info) = segment.find_symbol_by_vram_or_vrom(address) {
                return Some(info);
            }
        }

        None
    }

    pub fn find_symbol_by_vram(
        &self,
        address: u64,
    ) -> (
        Option<found_symbol_info::FoundSymbolInfo>,
        Vec<&section::Section>,
    ) {
        let mut possible_sections = Vec::new();

        for segment in &self.segments_list {
            let (maybe_info, possible_sections_aux) = segment.find_symbol_by_vram(address);
            if let Some(info) = maybe_info {
                return (Some(info), Vec::new());
            }
            possible_sections.extend(possible_sections_aux);
        }

        (None, possible_sections)
    }

    pub fn find_symbol_by_vrom(
        &self,
        address: u64,
    ) -> (
        Option<found_symbol_info::FoundSymbolInfo>,
        Vec<&section::Section>,
    ) {
        let mut possible_sections = Vec::new();

        for segment in &self.segments_list {
            let (maybe_info, possible_sections_aux) = segment.find_symbol_by_vrom(address);
            if let Some(info) = maybe_info {
                return (Some(info), Vec::new());
            }
            possible_sections.extend(possible_sections_aux);
        }

        (None, possible_sections)
    }

    pub fn find_lowest_differing_symbol(
        &self,
        other_map_file: &Self,
    ) -> Option<(&symbol::Symbol, &section::Section, Option<&symbol::Symbol>)> {
        let mut min_vram = u64::MAX;
        let mut found = None;
        let mut found_indices = (0, 0);

        for (i, built_segment) in self.segments_list.iter().enumerate() {
            for (j, built_file) in built_segment.sections_list.iter().enumerate() {
                for (k, built_sym) in built_file.symbols.iter().enumerate() {
                    if let Some(expected_sym_info) =
                        other_map_file.find_symbol_by_name(&built_sym.name)
                    {
                        let expected_sym = &expected_sym_info.symbol;

                        if built_sym.vram != expected_sym.vram && built_sym.vram < min_vram {
                            min_vram = built_sym.vram;

                            let prev_sym = if k > 0 {
                                Some(&built_file.symbols[k - 1])
                            } else {
                                None
                            };
                            found = Some((built_sym, built_file, prev_sym));
                            found_indices = (i as isize, j as isize);
                        }
                    }
                }
            }
        }

        if let Some((found_built_sym, found_built_file, prev_sym)) = found {
            if prev_sym.is_none() {
                // Previous symbol was not in the same section of the given
                // section, so we try to backtrack until we find any symbol.

                let (mut i, mut j) = found_indices;

                // We want to check the previous section, not the current one,
                // since we already know the current one doesn't have a symbol
                // preceding the one we found.
                j -= 1;

                'outer: while i >= 0 {
                    let built_segment = &self.segments_list[i as usize];

                    while j >= 0 {
                        let built_file = &built_segment.sections_list[j as usize];

                        if !built_file.symbols.is_empty() {
                            found = Some((
                                found_built_sym,
                                found_built_file,
                                built_file.symbols.last(),
                            ));
                            break 'outer;
                        }

                        j -= 1;
                    }

                    i -= 1;
                    if i >= 0 {
                        j = self.segments_list[i as usize].sections_list.len() as isize - 1;
                    }
                }
            }
        }

        found
    }

    pub fn mix_folders(&self) -> Self {
        let mut new_map_file = MapFile::new();

        for segment in &self.segments_list {
            new_map_file.segments_list.push(segment.mix_folders());
        }

        new_map_file
    }

    pub fn fixup_non_matching_symbols(&self) -> Self {
        let mut new_map_file = self.clone();

        new_map_file
            .segments_list
            .iter_mut()
            .for_each(|x| x.fixup_non_matching_symbols());

        new_map_file
    }

    pub fn get_progress(
        &self,
        path_decomp_settings: Option<&section::PathDecompSettings>,
        aliases: &HashMap<String, String>,
    ) -> (
        progress_stats::ProgressStats,
        HashMap<String, progress_stats::ProgressStats>,
    ) {
        let mut total_stats = progress_stats::ProgressStats::new();
        let mut progress_per_folder: HashMap<String, progress_stats::ProgressStats> =
            HashMap::new();

        for segment in &self.segments_list {
            for section in &segment.sections_list {
                if section.symbols.is_empty() {
                    continue;
                }

                let folder = {
                    let path_index = if let Some(path_decomp_settings) = path_decomp_settings {
                        path_decomp_settings.path_index
                    } else {
                        section.filepath.components().count().saturating_sub(1)
                    };

                    let temp = section
                        .filepath
                        .components()
                        .nth(path_index)
                        .unwrap()
                        .as_os_str()
                        .to_str()
                        .unwrap();
                    if let Some(alternative_folder) = aliases.get(temp) {
                        alternative_folder
                    } else {
                        temp
                    }
                };

                let folder_progress = progress_per_folder.entry(folder.to_string()).or_default();

                for sym_state in section.symbol_match_state_iter(path_decomp_settings) {
                    match sym_state {
                        symbol_decomp_state::SymbolDecompState::Decomped(sym) => {
                            let sym_size = sym.size as usize;

                            total_stats.decomped_size += sym_size;
                            folder_progress.decomped_size += sym_size;
                        }
                        symbol_decomp_state::SymbolDecompState::Undecomped(sym) => {
                            let sym_size = sym.size as usize;

                            total_stats.undecomped_size += sym_size;
                            folder_progress.undecomped_size += sym_size;
                        }
                    }
                }
            }
        }

        (total_stats, progress_per_folder)
    }

    /// Useful for finding bss reorders
    pub fn compare_files_and_symbols<'a>(
        &'a self,
        other_map_file: &'a Self,
        check_other_on_self: bool,
    ) -> maps_comparison_info::MapsComparisonInfo<'a> {
        let mut comp_info = maps_comparison_info::MapsComparisonInfo::new();

        for segment in &self.segments_list {
            for section in &segment.sections_list {
                for symbol in &section.symbols {
                    if let Some(found_sym_info) = other_map_file.find_symbol_by_name(&symbol.name) {
                        let comp = symbol_comparison_info::SymbolComparisonInfo::new(
                            symbol,
                            symbol.vram,
                            Some(section),
                            symbol.vram,
                            Some(found_sym_info.section),
                        );

                        if comp.diff() != Some(0) {
                            comp_info.bad_sections.insert(section);
                        }
                        comp_info.compared_list.push(comp);
                    } else {
                        comp_info.missing_sections.insert(section);
                        comp_info.compared_list.push(
                            symbol_comparison_info::SymbolComparisonInfo::new(
                                symbol,
                                symbol.vram,
                                Some(section),
                                u64::MAX,
                                None,
                            ),
                        );
                    }
                }
            }
        }

        if check_other_on_self {
            for segment in &other_map_file.segments_list {
                for section in &segment.sections_list {
                    for symbol in &section.symbols {
                        let found_sym_info = self.find_symbol_by_name(&symbol.name);

                        if found_sym_info.is_none() {
                            comp_info.missing_sections.insert(section);
                            comp_info.compared_list.push(
                                symbol_comparison_info::SymbolComparisonInfo::new(
                                    symbol,
                                    u64::MAX,
                                    None,
                                    symbol.vram,
                                    Some(section),
                                ),
                            );
                        }
                    }
                }
            }
        }

        comp_info
    }

    pub fn to_csv(&self, print_vram: bool, skip_without_symbols: bool) -> String {
        let mut ret = section::Section::to_csv_header(print_vram) + "\n";

        for segment in &self.segments_list {
            ret += &segment.to_csv(print_vram, skip_without_symbols);
        }

        ret
    }

    pub fn to_csv_symbols(&self) -> String {
        let mut ret = String::new();

        writeln!(ret, "Section,{}", symbol::Symbol::to_csv_header()).unwrap();

        for segment in &self.segments_list {
            ret += &segment.to_csv_symbols();
        }

        ret
    }

    pub fn print_as_csv(&self, print_vram: bool, skip_without_symbols: bool) {
        print!("{}", self.to_csv(print_vram, skip_without_symbols));
    }

    pub fn print_symbols_csv(&self) {
        print!("{}", self.to_csv_symbols());
    }
}

impl MapFile {
    // TODO: figure out if this is doing unnecessary copies or something
    fn preprocess_map_data_gnu(map_data: &str) -> String {
        // Skip the stuff we don't care about
        // Looking for this string will only work on English machines (or C locales)
        // but it doesn't matter much, because if this string is not found then the
        // parsing should still work, but just a bit slower because of the extra crap
        if let Some(aux_var) = map_data.find("\nLinker script and memory map") {
            if let Some(start_index) = map_data[aux_var + 1..].find('\n') {
                return map_data[aux_var + 1 + start_index + 1..].to_string();
            }
        }

        map_data.to_string()
    }
}

impl Default for MapFile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "python_bindings")]
#[allow(non_snake_case)]
pub(crate) mod python_bindings {
    use pyo3::prelude::*;

    use std::{collections::HashMap, path::PathBuf};

    use crate::{
        found_symbol_info, maps_comparison_info, progress_stats, section, segment, symbol,
    };

    #[pymethods]
    impl super::MapFile {
        #[new]
        fn py_new() -> Self {
            Self::new()
        }

        #[staticmethod]
        fn newFromMapFile(map_path: PathBuf) -> Self {
            Self::new_from_map_file(&map_path)
        }

        fn readMapFile(&mut self, map_path: PathBuf) {
            self.read_map_file(&map_path)
        }

        fn parseMapContents(&mut self, map_contents: &str) {
            self.parse_map_contents(map_contents)
        }

        fn parseMapContentsGNU(&mut self, map_contents: &str) {
            self.parse_map_contents_gnu(map_contents)
        }

        /**
        Parses the contents of a clang ld.lld map.

        The `mapContents` argument must contain the contents of a clang ld.lld mapfile.
        */
        #[pyo3(name = "parseMapContentsLLD")]
        fn parseMapContentsLLD(&mut self, map_contents: &str) {
            self.parse_map_contents_lld(map_contents)
        }

        fn filterBySectionType(&self, section_type: &str) -> Self {
            self.filter_by_section_type(section_type)
        }

        fn getEverySectionExceptSectionType(&self, section_type: &str) -> Self {
            self.get_every_section_except_section_type(section_type)
        }

        fn getEveryFileExceptSectionType(&self, section_type: &str) -> Self {
            self.getEverySectionExceptSectionType(section_type)
        }

        fn findSymbolByName(
            &self,
            sym_name: &str,
        ) -> Option<found_symbol_info::python_bindings::PyFoundSymbolInfo> {
            self.find_symbol_by_name(sym_name)
                .map(found_symbol_info::python_bindings::PyFoundSymbolInfo::from)
        }

        fn findSymbolByVramOrVrom(
            &self,
            address: u64,
        ) -> Option<found_symbol_info::python_bindings::PyFoundSymbolInfo> {
            #[allow(deprecated)]
            self.find_symbol_by_vram_or_vrom(address)
                .map(found_symbol_info::python_bindings::PyFoundSymbolInfo::from)
        }

        fn findSymbolByVram(
            &self,
            address: u64,
        ) -> (
            Option<found_symbol_info::python_bindings::PyFoundSymbolInfo>,
            Vec<section::Section>,
        ) {
            let (info, possible_files) = self.find_symbol_by_vram(address);
            (
                info.map(found_symbol_info::python_bindings::PyFoundSymbolInfo::from),
                possible_files.into_iter().cloned().collect(),
            )
        }

        fn findSymbolByVrom(
            &self,
            address: u64,
        ) -> (
            Option<found_symbol_info::python_bindings::PyFoundSymbolInfo>,
            Vec<section::Section>,
        ) {
            let (info, possible_files) = self.find_symbol_by_vrom(address);
            (
                info.map(found_symbol_info::python_bindings::PyFoundSymbolInfo::from),
                possible_files.into_iter().cloned().collect(),
            )
        }

        fn findLowestDifferingSymbol(
            &self,
            other_map_file: &Self,
        ) -> Option<(symbol::Symbol, section::Section, Option<symbol::Symbol>)> {
            if let Some((s, f, os)) = self.find_lowest_differing_symbol(other_map_file) {
                Some((s.clone(), f.clone(), os.cloned()))
            } else {
                None
            }
        }

        fn mixFolders(&self) -> Self {
            self.mix_folders()
        }

        fn fixupNonMatchingSymbols(&self) -> Self {
            self.fixup_non_matching_symbols()
        }

        #[pyo3(signature = (asm_path, nonmatchings, aliases=HashMap::new(), path_index=2, check_function_files=true))]
        fn getProgress(
            &self,
            asm_path: PathBuf,
            nonmatchings: PathBuf,
            aliases: HashMap<String, String>,
            path_index: usize,
            check_function_files: bool,
        ) -> (
            progress_stats::ProgressStats,
            HashMap<String, progress_stats::ProgressStats>,
        ) {
            let path_decomp_settings = section::PathDecompSettings {
                asm_path: &asm_path,
                nonmatchings: &nonmatchings,
                path_index,
                check_function_files,
            };

            self.get_progress(Some(&path_decomp_settings), &aliases)
        }

        #[pyo3(signature=(other_map_file, *, check_other_on_self=true))]
        fn compareFilesAndSymbols(
            &self,
            other_map_file: &Self,
            check_other_on_self: bool,
        ) -> maps_comparison_info::python_bindings::PyMapsComparisonInfo {
            self.compare_files_and_symbols(other_map_file, check_other_on_self)
                .into()
        }

        #[pyo3(signature=(print_vram=true, skip_without_symbols=true))]
        fn toCsv(&self, print_vram: bool, skip_without_symbols: bool) -> String {
            self.to_csv(print_vram, skip_without_symbols)
        }

        fn toCsvSymbols(&self) -> String {
            self.to_csv_symbols()
        }

        #[pyo3(signature=(print_vram=true, skip_without_symbols=true))]
        fn printAsCsv(&self, print_vram: bool, skip_without_symbols: bool) {
            self.print_as_csv(print_vram, skip_without_symbols)
        }

        fn printSymbolsCsv(&self) {
            self.print_symbols_csv()
        }

        fn copySegmentList(&self) -> Vec<segment::Segment> {
            self.segments_list.clone()
        }

        fn setSegmentList(&mut self, new_list: Vec<segment::Segment>) {
            self.segments_list = new_list;
        }

        fn appendSegment(&mut self, segment: segment::Segment) {
            self.segments_list.push(segment);
        }

        fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<SegmentVecIter>> {
            let iter = SegmentVecIter {
                inner: slf.segments_list.clone().into_iter(),
            };
            Py::new(slf.py(), iter)
        }

        fn __getitem__(&self, index: usize) -> segment::Segment {
            self.segments_list[index].clone()
        }

        fn __setitem__(&mut self, index: usize, element: segment::Segment) {
            self.segments_list[index] = element;
        }

        fn __len__(&self) -> usize {
            self.segments_list.len()
        }
    }

    #[pyclass]
    struct SegmentVecIter {
        inner: std::vec::IntoIter<segment::Segment>,
    }

    #[pymethods]
    impl SegmentVecIter {
        fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
            slf
        }

        fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<segment::Segment> {
            slf.inner.next()
        }
    }
}
