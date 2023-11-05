/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Write;
use std::path::PathBuf;

use regex::*;

use pyo3::prelude::*;

use crate::{
    file, found_symbol_info, maps_comparison_info, progress_stats, segment, symbol,
    symbol_comparison_info, utils,
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
#[pyclass(module = "mapfile_parser")]
pub struct MapFile {
    pub segments_list: Vec<segment::Segment>,

    #[pyo3(get, set)]
    debugging: bool,
}

#[pymethods]
impl MapFile {
    #[new]
    pub fn new() -> Self {
        MapFile {
            segments_list: Vec::new(),
            debugging: false,
        }
    }

    /**
    Opens the mapfile pointed by the `mapPath` argument and parses it.

    The format of the map will be guessed based on its contents.

    Currently supported map formats:
    - GNU ld
    - clang ld.lld
     */
    #[pyo3(name = "readMapFile")]
    pub fn read_map_file(&mut self, map_path: PathBuf) {
        let map_contents = utils::read_file_contents(&map_path);

        self.parse_map_contents(map_contents);
    }

    /**
    Parses the contents of the map.

    The `mapContents` argument must contain the contents of a mapfile.

    The format of the map will be guessed based on its contents.

    Currently supported mapfile formats:
    - GNU ld
    - clang ld.lld
    */
    #[pyo3(name = "parseMapContents")]
    pub fn parse_map_contents(&mut self, map_contents: String) {
        let regex_lld_header =
            Regex::new(r"\s+VMA\s+LMA\s+Size\s+Align\s+Out\s+In\s+Symbol").unwrap();

        if regex_lld_header.is_match(&map_contents) {
            self.parse_map_contents_lld(map_contents);
        } else {
            // GNU is the fallback
            self.parse_map_contents_gnu(map_contents);
        }
    }

    /**
    Parses the contents of a GNU ld map.

    The `mapContents` argument must contain the contents of a GNU ld mapfile.
     */
    #[pyo3(name = "parseMapContentsGNU")]
    pub fn parse_map_contents_gnu(&mut self, map_contents: String) {
        // TODO: maybe move somewhere else?
        let regex_file_data_entry = Regex::new(r"^\s+(?P<section>\.[^\s]+)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s+(?P<name>[^\s]+)$").unwrap();
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

        let mut in_file = false;

        let mut prev_line = "";
        for line in map_data.split('\n') {
            if in_file {
                if !line.starts_with("        ") {
                    in_file = false;
                } else if !regex_label.is_match(line) {
                    // Filter out jump table's labels

                    // Find symbols
                    if let Some(entry_match) = regex_function_entry.captures(line) {
                        // println!("{entry_match:?}");
                        let sym_name = &entry_match["name"];

                        if !BANNED_SYMBOL_NAMES.contains(&sym_name) {
                            let sym_vram = utils::parse_hex(&entry_match["vram"]);

                            let current_segment = temp_segment_list.last_mut().unwrap();
                            let current_file = current_segment.files_list.last_mut().unwrap();

                            current_file
                                .symbols
                                .push(symbol::Symbol::new_default(sym_name.into(), sym_vram));
                        }
                    }
                }
            }

            if !in_file {
                if let Some(file_entry_match) = regex_file_data_entry.captures(line) {
                    let filepath = std::path::PathBuf::from(&file_entry_match["name"]);
                    let vram = utils::parse_hex(&file_entry_match["vram"]);
                    let size = utils::parse_hex(&file_entry_match["size"]);
                    let section_type = &file_entry_match["section"];

                    if size > 0 {
                        in_file = true;
                        let current_segment = temp_segment_list.last_mut().unwrap();

                        current_segment.files_list.push(file::File::new_default(
                            filepath,
                            vram,
                            size,
                            section_type,
                        ));
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

                    if !current_segment.files_list.is_empty() {
                        let prev_file = current_segment.files_list.last().unwrap();
                        let mut name = prev_file.filepath.file_name().unwrap().to_owned();

                        name.push("__fill__");
                        filepath = prev_file.filepath.with_file_name(name);
                        vram = prev_file.vram + prev_file.size;
                        section_type = prev_file.section_type.clone();
                    }

                    current_segment.files_list.push(file::File::new_default(
                        filepath,
                        vram,
                        size,
                        &section_type,
                    ));
                }
            }

            prev_line = line;
        }

        for (i, segment) in temp_segment_list.iter_mut().enumerate() {
            if i == 0 && segment.is_placeholder() {
                // skip the dummy segment if it has no size, files or symbols
                continue;
            }

            let mut new_segment = segment.clone_no_filelist();

            let mut vrom_offset = segment.vrom;
            for file in segment.files_list.iter_mut() {
                let mut acummulated_size = 0;
                let symbols_count = file.symbols.len();
                let is_noload_section = file.is_noload_section();

                if file.is_placeholder() {
                    // drop placeholders
                    continue;
                }

                if file.vrom.is_some() {
                    vrom_offset = file.vrom.unwrap();
                }

                if !is_noload_section {
                    file.vrom = Some(vrom_offset);
                }

                if symbols_count > 0 {
                    let mut sym_vrom = vrom_offset;

                    // Calculate size of each symbol
                    for index in 0..symbols_count - 1 {
                        let next_sym_vram = file.symbols[index + 1].vram;
                        let sym = &mut file.symbols[index];

                        if index == 0 && sym.vram > file.vram {
                            // If the vram of the first symbol doesn't match the vram of the file
                            // it means the first(s) symbols were not emitted in the mapfile (static,
                            // jumptables, etc)
                            // We try to adjust the vrom to account for it.
                            sym_vrom += sym.vram - file.vram;
                        }

                        let sym_size = next_sym_vram - sym.vram;
                        acummulated_size += sym_size;

                        sym.size = Some(sym_size);

                        if !is_noload_section {
                            // Only set vrom of non bss variables
                            sym.vrom = Some(sym_vrom);
                            sym_vrom += sym_size;
                        }
                    }

                    // Calculate size of last symbol of the file
                    let sym = &mut file.symbols[symbols_count - 1];
                    let sym_size = file.size - acummulated_size;
                    sym.size = Some(sym_size);
                    if !is_noload_section {
                        sym.vrom = Some(sym_vrom);
                        //sym_vrom += sym_size;
                    }
                }

                if !is_noload_section {
                    vrom_offset += file.size;
                }

                new_segment.files_list.push(file.clone());
            }

            self.segments_list.push(new_segment);
        }
    }

    /**
    Parses the contents of a clang ld.lld map.

    The `mapContents` argument must contain the contents of a clang ld.lld mapfile.
     */
    #[pyo3(name = "parseMapContentsLLD")]
    pub fn parse_map_contents_lld(&mut self, map_contents: String) {
        let map_data = map_contents;

        // Every line starts with this information, so instead of duplicating it we put them on one single regex
        let regex_row_entry = Regex::new(r"^\s*(?P<vram>[0-9a-fA-F]+)\s+(?P<vrom>[0-9a-fA-F]+)\s+(?P<size>[0-9a-fA-F]+)\s+(?P<align>[0-9a-fA-F]+) ").unwrap();

        let regex_segment_entry = Regex::new(r"^(?P<name>[^\s]+)$").unwrap();
        let regex_fill = Regex::new(r"^\s+(?P<expr>\.\s*\+=\s*.+)$").unwrap();
        let regex_file_data_entry =
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
                    // Make a dummy file to handle pads (. += XX)

                    let mut filepath = std::path::PathBuf::new();
                    let mut section_type = "".to_owned();

                    let current_segment = temp_segment_list.last_mut().unwrap();

                    if !current_segment.files_list.is_empty() {
                        let prev_file = current_segment.files_list.last().unwrap();
                        let mut name = prev_file.filepath.file_name().unwrap().to_owned();

                        name.push("__fill__");
                        filepath = prev_file.filepath.with_file_name(name);
                        section_type = prev_file.section_type.clone();
                    }

                    let mut new_file = file::File::new_default(filepath, vram, size, &section_type);
                    if !utils::is_noload_section(&section_type) {
                        new_file.vrom = Some(vrom);
                    }
                    current_segment.files_list.push(new_file);
                } else if let Some(file_entry_match) = regex_file_data_entry.captures(subline) {
                    let filepath = std::path::PathBuf::from(&file_entry_match["name"]);
                    let section_type = &file_entry_match["section"];

                    if size > 0 {
                        let current_segment = temp_segment_list.last_mut().unwrap();

                        let mut new_file =
                            file::File::new_default(filepath, vram, size, section_type);
                        if !utils::is_noload_section(section_type) {
                            new_file.vrom = Some(vrom);
                        }
                        new_file.align = Some(align);

                        current_segment.files_list.push(new_file);
                    }
                } else if regex_label.is_match(subline) {
                    // pass
                } else if let Some(symbol_entry_match) = regex_symbol_entry.captures(subline) {
                    let name = &symbol_entry_match["name"];

                    if !BANNED_SYMBOL_NAMES.contains(&name) {
                        let current_segment = temp_segment_list.last_mut().unwrap();
                        let current_file = current_segment.files_list.last_mut().unwrap();

                        let mut new_symbol = symbol::Symbol::new_default(name.into(), vram);
                        if size > 0 {
                            new_symbol.size = Some(size);
                        }
                        if !current_file.is_noload_section() {
                            new_symbol.vrom = Some(vrom)
                        }
                        new_symbol.align = Some(align);

                        current_file.symbols.push(new_symbol);
                    }
                }
            }
        }

        for (i, segment) in temp_segment_list.iter_mut().enumerate() {
            if i == 0 && segment.is_placeholder() {
                // skip the dummy segment if it has no size, files or symbols
                continue;
            }

            let mut new_segment = segment.clone_no_filelist();

            for file in segment.files_list.iter_mut() {
                if file.is_placeholder() {
                    // drop placeholders
                    continue;
                }

                let mut acummulated_size = 0;
                let symbols_count = file.symbols.len();

                if symbols_count > 0 {
                    // Calculate the size of symbols that the map file did not report.
                    // usually asm symbols and not C ones

                    for index in 0..symbols_count - 1 {
                        let next_sym_vram = file.symbols[index + 1].vram;
                        let sym = &mut file.symbols[index];

                        let sym_size = next_sym_vram - sym.vram;
                        acummulated_size += sym_size;

                        if sym.size.is_none() {
                            sym.size = Some(sym_size);
                        }
                    }

                    // Calculate size of last symbol of the file
                    let sym = &mut file.symbols[symbols_count - 1];
                    if sym.size.is_none() {
                        let sym_size = file.size - acummulated_size;
                        sym.size = Some(sym_size);
                    }
                }

                new_segment.files_list.push(file.clone());
            }

            self.segments_list.push(new_segment);
        }
    }

    #[pyo3(name = "filterBySectionType")]
    pub fn filter_by_section_type(&self, section_type: &str) -> MapFile {
        let mut new_map_file = MapFile::new();

        new_map_file.debugging = self.debugging;

        for segment in &self.segments_list {
            let new_segment = segment.filter_by_section_type(section_type);

            if !new_segment.files_list.is_empty() {
                new_map_file.segments_list.push(new_segment);
            }
        }

        new_map_file
    }

    #[pyo3(name = "getEveryFileExceptSectionType")]
    pub fn get_every_file_except_section_type(&self, section_type: &str) -> MapFile {
        let mut new_map_file = MapFile::new();

        new_map_file.debugging = self.debugging;

        for segment in &self.segments_list {
            let new_segment = segment.get_every_file_except_section_type(section_type);

            if !new_segment.files_list.is_empty() {
                new_map_file.segments_list.push(new_segment);
            }
        }

        new_map_file
    }

    #[pyo3(name = "findSymbolByName")]
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

    #[pyo3(name = "findSymbolByVramOrVrom")]
    pub fn find_symbol_by_vram_or_vrom(
        &self,
        address: u64,
    ) -> Option<found_symbol_info::FoundSymbolInfo> {
        for segment in &self.segments_list {
            if let Some(info) = segment.find_symbol_by_vram_or_vrom(address) {
                return Some(info);
            }
        }

        None
    }

    #[pyo3(name = "findLowestDifferingSymbol")]
    pub fn find_lowest_differing_symbol(
        &self,
        other_map_file: MapFile,
    ) -> Option<(symbol::Symbol, file::File, Option<symbol::Symbol>)> {
        let mut min_vram = u64::MAX;
        let mut found = None;

        for built_segement in &self.segments_list {
            for built_file in &built_segement.files_list {
                for (i, built_sym) in built_file.symbols.iter().enumerate() {
                    if let Some(expected_sym_info) =
                        other_map_file.find_symbol_by_name(&built_sym.name)
                    {
                        let expected_sym = &expected_sym_info.symbol;

                        if built_sym.vram != expected_sym.vram && built_sym.vram < min_vram {
                            min_vram = built_sym.vram;

                            let mut prev_sym = None;
                            if i > 0 {
                                prev_sym = Some(built_file.symbols[i - 1].clone());
                            }
                            found = Some((built_sym, built_file, prev_sym));
                        }
                    }
                }
            }
        }

        if let Some(found_temp) = found {
            return Some((found_temp.0.clone(), found_temp.1.clone(), found_temp.2));
        }
        None
    }

    #[pyo3(name = "mixFolders")]
    pub fn mix_folders(&self) -> MapFile {
        let mut new_map_file = MapFile::new();

        new_map_file.debugging = self.debugging;

        for segment in &self.segments_list {
            new_map_file.segments_list.push(segment.mix_folders());
        }

        new_map_file
    }

    #[pyo3(name = "getProgress", signature = (asm_path, nonmatchings, aliases=HashMap::new(), path_index=2))]
    pub fn get_progress(
        &self,
        asm_path: PathBuf,
        nonmatchings: PathBuf,
        aliases: HashMap<String, String>,
        path_index: usize,
    ) -> (
        progress_stats::ProgressStats,
        HashMap<String, progress_stats::ProgressStats>,
    ) {
        let mut total_stats = progress_stats::ProgressStats::new();
        let mut progress_per_folder: HashMap<String, progress_stats::ProgressStats> =
            HashMap::new();

        for segment in &self.segments_list {
            for file in &segment.files_list {
                if file.symbols.is_empty() {
                    continue;
                }

                let mut folder = &file
                    .filepath
                    .components()
                    .nth(path_index)
                    .unwrap()
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .to_string();
                if let Some(alternative_folder) = aliases.get(folder) {
                    folder = alternative_folder;
                }

                if !progress_per_folder.contains_key(folder) {
                    progress_per_folder
                        .insert(folder.clone(), progress_stats::ProgressStats::new());
                }
                let folder_progress = progress_per_folder.get_mut(folder).unwrap();

                let original_file_path: PathBuf =
                    file.filepath.components().skip(path_index).collect();

                let mut extensionless_file_path = original_file_path;
                while extensionless_file_path.extension().is_some() {
                    extensionless_file_path.set_extension("");
                }

                let full_asm_file = asm_path.join(extensionless_file_path.with_extension("s"));
                let whole_file_is_undecomped = full_asm_file.exists();

                for func in &file.symbols {
                    let func_asm_path = nonmatchings
                        .join(extensionless_file_path.clone())
                        .join(func.name.clone() + ".s");

                    let sym_size = func.size.unwrap_or(0) as u32;

                    if whole_file_is_undecomped || func_asm_path.exists() {
                        total_stats.undecomped_size += sym_size;
                        folder_progress.undecomped_size += sym_size;
                    } else {
                        total_stats.decomped_size += sym_size;
                        folder_progress.decomped_size += sym_size;
                    }
                }
            }
        }

        (total_stats, progress_per_folder)
    }

    #[pyo3(name = "compareFilesAndSymbols", signature=(other_map_file, *, check_other_on_self=true))]
    /// Useful for finding bss reorders
    pub fn compare_files_and_symbols(
        &self,
        other_map_file: MapFile,
        check_other_on_self: bool,
    ) -> maps_comparison_info::MapsComparisonInfo {
        let mut comp_info = maps_comparison_info::MapsComparisonInfo::new();

        for segment in &self.segments_list {
            for file in &segment.files_list {
                for symbol in &file.symbols {
                    if let Some(found_sym_info) = other_map_file.find_symbol_by_name(&symbol.name) {
                        let diff = symbol.vram as i64 - found_sym_info.symbol.vram as i64;
                        let comp = symbol_comparison_info::SymbolComparisonInfo::new(
                            symbol.clone(),
                            symbol.vram,
                            Some(file.clone()),
                            symbol.vram,
                            Some(found_sym_info.file),
                            Some(diff),
                        );

                        if diff != 0 {
                            comp_info.bad_files.insert(file.clone());
                        }
                        comp_info.compared_list.push(comp);
                    } else {
                        comp_info.missing_files.insert(file.clone());
                        comp_info.compared_list.push(
                            symbol_comparison_info::SymbolComparisonInfo::new(
                                symbol.clone(),
                                symbol.vram,
                                Some(file.clone()),
                                u64::MAX,
                                None,
                                None,
                            ),
                        );
                    }
                }
            }
        }

        if check_other_on_self {
            for segment in &other_map_file.segments_list {
                for file in &segment.files_list {
                    for symbol in &file.symbols {
                        let found_sym_info = self.find_symbol_by_name(&symbol.name);

                        if found_sym_info.is_none() {
                            comp_info.missing_files.insert(file.clone());
                            comp_info.compared_list.push(
                                symbol_comparison_info::SymbolComparisonInfo::new(
                                    symbol.clone(),
                                    u64::MAX,
                                    None,
                                    symbol.vram,
                                    Some(file.clone()),
                                    None,
                                ),
                            );
                        }
                    }
                }
            }
        }

        comp_info
    }

    #[pyo3(name = "toCsv", signature=(print_vram=true, skip_without_symbols=true))]
    pub fn to_csv(&self, print_vram: bool, skip_without_symbols: bool) -> String {
        let mut ret = file::File::to_csv_header(print_vram) + "\n";

        for segment in &self.segments_list {
            ret += &segment.to_csv(print_vram, skip_without_symbols);
        }

        ret
    }

    #[pyo3(name = "toCsvSymbols")]
    pub fn to_csv_symbols(&self) -> String {
        let mut ret = String::new();

        writeln!(ret, "File,{}", symbol::Symbol::to_csv_header()).unwrap();

        for segment in &self.segments_list {
            ret += &segment.to_csv_symbols();
        }

        ret
    }

    #[pyo3(name = "printAsCsv", signature=(print_vram=true, skip_without_symbols=true))]
    pub fn print_as_csv(&self, print_vram: bool, skip_without_symbols: bool) {
        print!("{}", self.to_csv(print_vram, skip_without_symbols));
    }

    #[pyo3(name = "printSymbolsCsv")]
    pub fn print_symbols_csv(&self) {
        print!("{}", self.to_csv_symbols());
    }

    #[cfg(feature = "python_bindings")]
    #[pyo3(name = "copySegmentList")]
    fn copy_segment_list(&self) -> Vec<segment::Segment> {
        self.segments_list.clone()
    }

    #[cfg(feature = "python_bindings")]
    #[pyo3(name = "setSegmentList")]
    fn set_segment_list(&mut self, new_list: Vec<segment::Segment>) {
        self.segments_list = new_list;
    }

    #[cfg(feature = "python_bindings")]
    #[pyo3(name = "appendSegment")]
    fn append_segment(&mut self, segment: segment::Segment) {
        self.segments_list.push(segment);
    }

    #[cfg(feature = "python_bindings")]
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<SegmentVecIter>> {
        let iter = SegmentVecIter {
            inner: slf.segments_list.clone().into_iter(),
        };
        Py::new(slf.py(), iter)
    }

    #[cfg(feature = "python_bindings")]
    fn __getitem__(&self, index: usize) -> segment::Segment {
        self.segments_list[index].clone()
    }

    #[cfg(feature = "python_bindings")]
    fn __setitem__(&mut self, index: usize, element: segment::Segment) {
        self.segments_list[index] = element;
    }

    #[cfg(feature = "python_bindings")]
    fn __len__(&self) -> usize {
        self.segments_list.len()
    }
}

impl MapFile {
    // TODO: figure out if this is doing unnecessary copies or something
    fn preprocess_map_data_gnu(mut map_data: String) -> String {
        // Skip the stuff we don't care about
        // Looking for this string will only work on English machines (or C locales)
        // but it doesn't matter much, because if this string is not found then the
        // parsing should still work, but just a bit slower because of the extra crap
        if let Some(aux_var) = map_data.find("\nLinker script and memory map") {
            if let Some(start_index) = map_data[aux_var + 1..].find('\n') {
                map_data = map_data[aux_var + 1 + start_index + 1..].to_string();
            }
        }

        map_data
    }
}

impl Default for MapFile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "python_bindings")]
#[pyclass]
struct SegmentVecIter {
    inner: std::vec::IntoIter<segment::Segment>,
}

#[cfg(feature = "python_bindings")]
#[pymethods]
impl SegmentVecIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<segment::Segment> {
        slf.inner.next()
    }
}
