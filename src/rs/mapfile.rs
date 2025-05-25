/* SPDX-FileCopyrightText: © 2023-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::{collections::HashMap, fmt::Write};

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    found_symbol_info, maps_comparison_info, progress_stats, section, segment, symbol,
    symbol_comparison_info, symbol_decomp_state,
};

#[derive(Debug, Clone)]
#[non_exhaustive]
// TODO: sequence?
#[cfg_attr(feature = "python_bindings", pyclass(module = "mapfile_parser"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MapFile {
    pub segments_list: Vec<segment::Segment>,

    #[cfg(feature = "python_bindings")]
    #[pyo3(get, set)]
    pub(crate) debugging: bool,
}

impl MapFile {
    pub fn filter_by_section_type(&self, section_type: &str) -> Self {
        let mut new_map_file = MapFile::new_impl();

        for segment in &self.segments_list {
            let new_segment = segment.filter_by_section_type(section_type);

            if !new_segment.sections_list.is_empty() {
                new_map_file.segments_list.push(new_segment);
            }
        }

        new_map_file
    }

    pub fn get_every_section_except_section_type(&self, section_type: &str) -> Self {
        let mut new_map_file = MapFile::new_impl();

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
        let mut new_map_file = MapFile::new_impl();

        for segment in &self.segments_list {
            new_map_file.segments_list.push(segment.mix_folders());
        }

        new_map_file
    }

    #[deprecated(
        since = "2.8.0",
        note = "This functionality is perform automatically during parsing now."
    )]
    pub fn fixup_non_matching_symbols(&self) -> Self {
        // This is a no-op now
        self.clone()
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

#[cfg(feature = "python_bindings")]
#[allow(non_snake_case)]
pub(crate) mod python_bindings {
    use pyo3::prelude::*;

    use std::{
        collections::HashMap,
        fs,
        io::{self, BufWriter},
        path::PathBuf,
    };

    use crate::{
        found_symbol_info, maps_comparison_info, progress_stats, report::ReportCategories, section,
        segment, symbol,
    };

    #[pymethods]
    impl super::MapFile {
        #[new]
        fn py_new() -> Self {
            Self::new_impl()
        }

        #[staticmethod]
        fn newFromMapFile(map_path: PathBuf) -> Self {
            Self::new_from_map_file(&map_path)
        }

        #[staticmethod]
        fn newFromMapStr(map_contents: &str) -> Self {
            Self::new_from_map_str(map_contents)
        }

        #[staticmethod]
        fn newFromGnuMapStr(map_contents: &str) -> Self {
            Self::new_from_gnu_map_str(map_contents)
        }

        #[staticmethod]
        fn newFromLldMapStr(map_contents: &str) -> Self {
            Self::new_from_lld_map_str(map_contents)
        }

        #[staticmethod]
        fn newFromMwMapStr(map_contents: &str) -> Self {
            Self::new_from_mw_map_str(map_contents)
        }

        fn readMapFile(&mut self, map_path: PathBuf) {
            #[allow(deprecated)]
            self.read_map_file(&map_path)
        }

        fn parseMapContents(&mut self, map_contents: &str) {
            #[allow(deprecated)]
            self.parse_map_contents(map_contents)
        }

        fn parseMapContentsGNU(&mut self, map_contents: &str) {
            #[allow(deprecated)]
            self.parse_map_contents_gnu(map_contents)
        }

        /**
        Parses the contents of a clang ld.lld map.

        The `mapContents` argument must contain the contents of a clang ld.lld mapfile.
        */
        #[pyo3(name = "parseMapContentsLLD")]
        fn parseMapContentsLLD(&mut self, map_contents: &str) {
            #[allow(deprecated)]
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
            #[allow(deprecated)]
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
                path_index,
                nonmatchings: check_function_files.then_some(nonmatchings.as_path()),
            };

            self.get_progress(Some(&path_decomp_settings), &aliases)
        }

        #[pyo3(signature = (outpath, prefixes_to_trim, report_categories, pathIndex=2, asmPath=None, nonmatchingsPath=None))]
        fn writeObjdiffReportToFile(
            &self,
            outpath: PathBuf,
            prefixes_to_trim: Vec<String>,
            report_categories: ReportCategories,
            pathIndex: usize,
            asmPath: Option<PathBuf>,
            nonmatchingsPath: Option<PathBuf>,
        ) -> Result<(), io::Error> {
            let path_decomp_settings = asmPath.as_ref().map(|x| section::PathDecompSettings {
                asm_path: x,
                path_index: pathIndex,
                nonmatchings: nonmatchingsPath.as_deref(),
            });

            let report = self.get_objdiff_report(
                report_categories,
                path_decomp_settings.as_ref(),
                |section| {
                    let mut section_name = section.filepath.to_string_lossy().to_string();
                    // Trim the first prefix found.
                    for x in &prefixes_to_trim {
                        if section_name.starts_with(x) {
                            section_name = section_name
                                .trim_start_matches(x)
                                .trim_matches('/')
                                .to_string();
                            break;
                        }
                    }
                    // Trim extensions
                    for x in [".s.o", ".c.o", ".cpp.o", ".o"] {
                        if section_name.ends_with(x) {
                            section_name = section_name.trim_end_matches(x).to_string();
                            break;
                        }
                    }
                    section_name
                },
            );

            // Stolen code from `objdiff` (objdiff-cli/src/util/output.rs)
            let file = fs::File::options()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(outpath)?;

            let mut buf = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut buf, &report)?;

            Ok(())
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
