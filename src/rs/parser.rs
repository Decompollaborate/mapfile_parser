/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    path::{Path, PathBuf},
};

use regex::*;

use crate::{mapfile::MapFile, section, segment, symbol, utils, Section};

lazy_static! {
    static ref BANNED_SYMBOL_NAMES: HashSet<&'static str> = {
        let mut symbol_names = HashSet::new();
        symbol_names.insert("gcc2_compiled.");
        symbol_names
    };
}

// TODO: Change all the deprecated functions to private and undeprecate them in a future version.

impl MapFile {
    /// Creates a new `MapFile` with the contents from the file pointed by the
    /// `map_path` argument.
    ///
    /// The format of the map will be guessed based on its contents.
    ///
    /// Currently supported map formats:
    /// - GNU ld
    /// - clang ld.lld
    /// - Metrowerks ld
    #[must_use]
    pub fn new_from_map_file(map_path: &Path) -> Self {
        let mut m = Self::new_impl();
        #[allow(deprecated)]
        m.read_map_file(map_path);
        m
    }

    /// Creates a new `MapFile` by parsing the contents of the map.
    ///
    /// The format of the map will be guessed based on its contents.
    ///
    /// Currently supported map formats:
    /// - GNU ld
    /// - clang ld.lld
    /// - Metrowerks ld
    #[must_use]
    pub fn new_from_map_str(map_contents: &str) -> Self {
        let mut m = Self::new_impl();
        #[allow(deprecated)]
        m.parse_map_contents(map_contents);
        m
    }

    /// Parses the contents of a GNU ld map.
    ///
    /// The `map_contents` argument must contain the contents of a GNU ld mapfile.
    #[must_use]
    pub fn new_from_gnu_map_str(map_contents: &str) -> Self {
        let mut m = Self::new_impl();
        #[allow(deprecated)]
        m.parse_map_contents_gnu(map_contents);
        m
    }

    /// Parses the contents of a clang ld.lld map.
    ///
    /// The `map_contents` argument must contain the contents of a clang ld.lld mapfile.
    #[must_use]
    pub fn new_from_lld_map_str(map_contents: &str) -> Self {
        let mut m = Self::new_impl();
        #[allow(deprecated)]
        m.parse_map_contents_lld(map_contents);
        m
    }

    /// Parses the contents of a Metrowerks ld (mwld) map.
    ///
    /// The `map_contents` argument must contain the contents of a Metrowerks ld mapfile.
    #[must_use]
    pub fn new_from_mw_map_str(map_contents: &str) -> Self {
        let mut m = Self::new_impl();
        m.parse_map_contents_mw(map_contents);
        m
    }

    pub(crate) fn new_impl() -> Self {
        Self {
            segments_list: Vec::new(),

            #[cfg(feature = "python_bindings")]
            debugging: false,
        }
    }

    #[deprecated(
        since = "2.8.0",
        note = "Use either `new_from_map_file` or `new_from_map_str` instead."
    )]
    pub fn new() -> Self {
        Self::new_impl()
    }

    /**
    Opens the mapfile pointed by the `map_path` argument and parses it.

    The format of the map will be guessed based on its contents.

    Currently supported map formats:
    - GNU ld
    - clang ld.lld
    - Metrowerks ld
     */
    #[deprecated(since = "2.8.0", note = "Prefer `MapFile::new_from_map_file` instead")]
    pub fn read_map_file(&mut self, map_path: &Path) {
        let map_contents = utils::read_file_contents(map_path);

        #[allow(deprecated)]
        self.parse_map_contents(&map_contents);
    }

    /**
    Parses the contents of the map.

    The `map_contents` argument must contain the contents of a mapfile.

    The format of the map will be guessed based on its contents.

    Currently supported mapfile formats:
    - GNU ld
    - clang ld.lld
    - Metrowerks ld
    */
    #[deprecated(since = "2.8.0", note = "Prefer `MapFile::new_from_map_str` instead")]
    pub fn parse_map_contents(&mut self, map_contents: &str) {
        let regex_lld_header =
            Regex::new(r"\s+VMA\s+LMA\s+Size\s+Align\s+Out\s+In\s+Symbol").unwrap();

        if regex_lld_header.is_match(map_contents) {
            #[allow(deprecated)]
            self.parse_map_contents_lld(map_contents);
        } else if map_contents.starts_with("Link map of ")
            || map_contents.contains(" section layout")
        {
            self.parse_map_contents_mw(map_contents);
        } else {
            // GNU is the fallback
            #[allow(deprecated)]
            self.parse_map_contents_gnu(map_contents);
        }
    }
}

impl MapFile {
    /**
    Parses the contents of a GNU ld map.

    The `map_contents` argument must contain the contents of a GNU ld mapfile.
     */
    #[deprecated(
        since = "2.8.0",
        note = "Prefer `MapFile::new_from_gnu_map_str` instead"
    )]
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
        let regex_romless_segment_entry =
            Regex::new(r"(?P<name>([^\s]+)?)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)$").unwrap();

        let map_data = MapFile::preprocess_map_data_gnu(map_contents);

        let mut temp_segment_list = vec![segment::Segment::new_placeholder()];

        let mut in_section = false;

        let mut prev_line = "";
        for line in map_data.lines() {
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
                    let filepath = PathBuf::from(&section_entry_match["name"]);
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
                    let vrom = Some(utils::parse_hex(&segment_entry_match["vrom"]));

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
                } else if let Some(segment_entry_match) = regex_romless_segment_entry.captures(line)
                {
                    // Some segments do not have a rom address
                    let mut name = &segment_entry_match["name"];
                    let vram = utils::parse_hex(&segment_entry_match["vram"]);
                    let size = utils::parse_hex(&segment_entry_match["size"]);
                    let vrom = None;

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
                    let mut filepath = PathBuf::new();
                    let mut vram = 0;
                    let size = utils::parse_hex(&fill_match["size"]);
                    let mut section_type = "".to_owned();

                    let current_segment = temp_segment_list.last_mut().unwrap();

                    if !current_segment.sections_list.is_empty() {
                        let prev_section = current_segment.sections_list.last().unwrap();
                        let mut name = prev_section
                            .filepath
                            .file_name()
                            .unwrap_or_else(|| OsStr::new(""))
                            .to_owned();

                        name.push("__fill__");
                        filepath = prev_section.filepath.with_file_name(name);
                        vram = prev_section.vram + prev_section.size;
                        section_type.clone_from(&prev_section.section_type);
                    }

                    current_segment
                        .sections_list
                        .push(section::Section::new_fill(
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

        // We need to keep a calculated rom in case the segment doesn't specify it explicitly
        let mut current_calculated_section_rom = 0;

        for (i, segment) in temp_segment_list.into_iter().enumerate() {
            if i == 0 && segment.is_placeholder() {
                // skip the dummy segment if it has no size, sections or symbols
                continue;
            }
            if segment.size == 0 && segment.sections_list.is_empty() {
                // Drop empty segments
                continue;
            }

            let mut new_segment = segment.clone_no_sectionlist();

            let mut vrom_offset = if let Some(vrom) = segment.vrom {
                current_calculated_section_rom = vrom;
                vrom
            } else {
                new_segment.vrom = Some(current_calculated_section_rom);
                current_calculated_section_rom
            };
            for mut section in segment.sections_list.into_iter() {
                if section.is_placeholder() {
                    // drop placeholders
                    continue;
                }

                // The size of the section
                let mut acummulated_size = 0;
                let symbols_count = section.symbols.len();
                let is_noload_section = section.is_noload_section();

                if let Some(vrom) = section.vrom {
                    vrom_offset = vrom;
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
                        sym_vrom = sym_vrom + first_sym.vram - section.vram;

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

                    Self::fixup_non_matching_symbols_for_section(&mut section);
                }

                if !is_noload_section {
                    vrom_offset += section.size;
                    current_calculated_section_rom += section.size;
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
    #[deprecated(
        since = "2.8.0",
        note = "Prefer `MapFile::new_from_lld_map_str` instead"
    )]
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

        for line in map_data.lines() {
            if let Some(row_entry_match) = regex_row_entry.captures(line) {
                let vram = utils::parse_hex(&row_entry_match["vram"]);
                let vrom = Some(utils::parse_hex(&row_entry_match["vrom"]));
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

                    let mut filepath = PathBuf::new();
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
                        section::Section::new_fill(filepath, vram, size, &section_type);
                    if !utils::is_noload_section(&section_type) {
                        new_section.vrom = vrom;
                    }
                    current_segment.sections_list.push(new_section);
                } else if let Some(section_entry_match) = regex_section_data_entry.captures(subline)
                {
                    let filepath = PathBuf::from(&section_entry_match["name"]);
                    let section_type = &section_entry_match["section"];

                    if size > 0 {
                        let current_segment = temp_segment_list.last_mut().unwrap();

                        let mut new_section =
                            section::Section::new_default(filepath, vram, size, section_type);
                        if !utils::is_noload_section(section_type) {
                            new_section.vrom = vrom;
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
                            new_symbol.vrom = vrom
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

                    Self::fixup_non_matching_symbols_for_section(&mut section);
                }

                new_segment.sections_list.push(section);
            }

            segments_list.push(new_segment);
        }

        segments_list.shrink_to_fit();
        segments_list
    }

    fn parse_map_contents_mw(&mut self, map_contents: &str) {
        let map_data = preprocess_map_data_mw(map_contents);

        let memory_map = parse_memory_map_mw(map_data);

        // Almost every line starts with this information, so instead of duplicating it we put them on one single regex
        let regex_row_entry = Regex::new(r"^\s*(?P<starting>[0-9a-fA-F]+)\s+(?P<size>[0-9a-fA-F]+)\s+(?P<vram>[0-9a-fA-F]+)\s+(?P<align>[0-9a-fA-F]+)\s+(?P<subline>.+)").unwrap();

        let regex_segment_entry = Regex::new(r"^(?P<name>.+) section layout$").unwrap();
        let regex_label_entry =
            Regex::new(r"^(?P<label>lbl_[0-9A-F]{8})\s+(?P<filename>.+?)\s*$").unwrap();
        let regex_symbol_entry =
            Regex::new(r"^\s*(?P<name>[^ ]+)\s+(?P<filename>.+?)\s*$").unwrap();

        let mut temp_segment_list = vec![segment::Segment::new_placeholder()];

        // Use a bunch of characters that shouldn't be valid in any os as a marker that we haven't found a file yet.
        let mut current_filename = "invalid file <>:\"/\\|?*".to_string();

        for line in map_data.lines() {
            // Check for regex_row_entry since it is more likely to match
            if let Some(row_entry_match) = regex_row_entry.captures(line) {
                let starting = utils::parse_hex(&row_entry_match["starting"]);
                let size = utils::parse_hex(&row_entry_match["size"]);
                let vram = utils::parse_hex(&row_entry_match["vram"]);
                let align = utils::parse_hex(&row_entry_match["align"]);

                let subline = &row_entry_match["subline"];

                if regex_label_entry.is_match(subline) {
                    // pass
                } else if let Some(symbol_entry_match) = regex_symbol_entry.captures(subline) {
                    let filename = &symbol_entry_match["filename"];

                    if filename == current_filename {
                        // We are still in the same file
                        let symbol = &symbol_entry_match["name"];

                        if !BANNED_SYMBOL_NAMES.contains(&symbol) {
                            let current_segment = temp_segment_list.last_mut().unwrap();
                            let current_section = current_segment.sections_list.last_mut().unwrap();

                            let mut new_symbol =
                                symbol::Symbol::new_default(symbol.to_string(), vram);
                            if size > 0 {
                                new_symbol.size = size;
                            }
                            if !current_section.is_noload_section() {
                                new_symbol.vrom = current_segment.vrom.map(|x| x + starting)
                            }
                            if align > 0 {
                                new_symbol.align = Some(align);
                            }

                            current_section.symbols.push(new_symbol);
                        }
                    } else {
                        // New file!
                        current_filename = filename.to_string();

                        if size > 0 {
                            let section_type = &symbol_entry_match["name"];
                            let filepath = PathBuf::from(filename);

                            let current_segment = temp_segment_list.last_mut().unwrap();

                            let mut new_section =
                                section::Section::new_default(filepath, vram, size, section_type);
                            if !utils::is_noload_section(section_type) {
                                new_section.vrom = current_segment.vrom.map(|x| x + starting)
                            }

                            current_segment.sections_list.push(new_section);
                        }
                    }
                } else {
                    println!("{}", subline);
                }
            } else if let Some(segment_entry_match) = regex_segment_entry.captures(line) {
                let name = &segment_entry_match["name"];

                let new_segment = if let Some(segment_entry) = memory_map.get(name) {
                    let vram = segment_entry.starting_address;
                    let size = segment_entry.size;
                    let vrom = Some(segment_entry.file_offset);
                    segment::Segment::new_default(name.to_string(), vram, size, vrom)
                } else {
                    let mut temp = segment::Segment::new_placeholder();
                    temp.name = name.to_string();
                    temp
                };

                temp_segment_list.push(new_segment);
            }
        }

        self.segments_list = Self::post_process_segments_mw(temp_segment_list);
    }

    fn post_process_segments_mw(temp_segment_list: Vec<segment::Segment>) -> Vec<segment::Segment> {
        // TODO: actually implement
        let mut segments_list = Vec::with_capacity(temp_segment_list.len());

        for (i, segment) in temp_segment_list.into_iter().enumerate() {
            if i == 0 && (segment.sections_list.is_empty() || segment.is_placeholder()) {
                // skip the dummy segment if it has no size, sections or symbols
                continue;
            }

            let mut new_segment = segment.clone_no_sectionlist();

            for mut section in segment.sections_list.into_iter() {
                if section.is_placeholder() {
                    // drop placeholders
                    continue;
                }

                let symbols_count = section.symbols.len();
                if symbols_count > 0 {
                    Self::fixup_non_matching_symbols_for_section(&mut section);
                }

                new_segment.sections_list.push(section);
            }

            segments_list.push(new_segment);
        }

        segments_list.shrink_to_fit();
        segments_list
    }
}

fn preprocess_map_data_mw(map_data: &str) -> &str {
    // Skip the stuff we don't care about.
    if let Some(aux_var) = map_data.find(" section layout") {
        // We want to preserve the name of the first layout, so we need to
        // backtrack a bit to find the start of the line
        if let Some(start_index) = map_data[..=aux_var].rfind("\n") {
            return &map_data[start_index + 1..];
        }
    }

    map_data
}

struct MwMemoryMapEntry {
    starting_address: u64,
    size: u64,
    file_offset: u64,
}

fn parse_memory_map_mw(map_data: &str) -> HashMap<String, MwMemoryMapEntry> {
    let map_data = {
        if let Some(start_index) = map_data.find("Memory map:") {
            if let Some(end_index) = map_data[start_index..].find("Linker generated symbols:") {
                &map_data[start_index..start_index + end_index]
            } else {
                &map_data[start_index..]
            }
        } else {
            map_data
        }
    };

    let mut memory_map = HashMap::new();
    let entry = Regex::new(r"^\s*(?P<name>[^ ]+)\s+(?P<address>[0-9a-fA-F]+)\s+(?P<size>[0-9a-fA-F]+)\s+(?P<offset>[0-9a-fA-F]+)$").unwrap();

    for line in map_data.lines() {
        if let Some(entry_match) = entry.captures(line) {
            let name = &entry_match["name"];
            let starting_address = utils::parse_hex(&entry_match["address"]);
            let size = utils::parse_hex(&entry_match["size"]);
            let file_offset = utils::parse_hex(&entry_match["offset"]);

            memory_map.insert(
                name.to_string(),
                MwMemoryMapEntry {
                    starting_address,
                    size,
                    file_offset,
                },
            );
        }
    }

    memory_map
}

impl MapFile {
    fn preprocess_map_data_gnu(map_data: &str) -> &str {
        // Skip the stuff we don't care about
        // Looking for this string will only work on English machines (or C locales)
        // but it doesn't matter much, because if this string is not found then the
        // parsing should still work, but just a bit slower because of the extra crap
        if let Some(aux_var) = map_data.find("\nLinker script and memory map") {
            if let Some(start_index) = map_data[aux_var + 1..].find('\n') {
                return &map_data[aux_var + 1 + start_index + 1..];
            }
        }

        map_data
    }

    fn fixup_non_matching_symbols_for_section(section: &mut Section) {
        // Fixup `.NON_MATCHING` symbols.
        // These kind of symbols have the same address as their
        // real counterpart, but their order is not guaranteed,
        // meaning we may have set the symbol size's to the
        // non_matching placeholder instead of the actual symbol.
        let mut nonmatchings_syms_original = Vec::new();
        let mut nonmatchings_syms_suffix = Vec::new();
        for (index, sym) in section.symbols.iter().enumerate() {
            if sym.name.ends_with(".NON_MATCHING") {
                let real_name = sym.name.replace(".NON_MATCHING", "");

                if let Some((real_sym, real_index)) =
                    section.find_symbol_and_index_by_name(&real_name)
                {
                    // One of the sizes should be zero, while the
                    // other non-zero, so we take the largest.
                    nonmatchings_syms_original.push((real_index, sym.size.max(real_sym.size)));
                    nonmatchings_syms_suffix.push(index);
                }
            }
        }
        for (index, new_size) in nonmatchings_syms_original {
            if let Some(sym) = section.symbols.get_mut(index) {
                sym.size = new_size;
                sym.nonmatching_sym_exists = true;
            }
        }
        for index in nonmatchings_syms_suffix {
            if let Some(sym) = section.symbols.get_mut(index) {
                sym.size = 0;
            }
        }
    }
}
