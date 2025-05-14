/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::HashSet;

use crate::{
    file::{self, PathDecompSettings},
    mapfile,
};
use objdiff_core::bindings::report;

impl mapfile::MapFile {
    #[must_use]
    pub fn get_objdiff_report(
        &self,
        path_decomp_settings: Option<&PathDecompSettings>,
    ) -> report::Report {
        do_report(self, path_decomp_settings)
    }
}

fn do_report(
    mapfile: &mapfile::MapFile,
    path_decomp_settings: Option<&PathDecompSettings>,
) -> report::Report {
    let mut units: Vec<report::ReportUnit> = Vec::new();
    let mut progress_categories = HashSet::new();

    for (segment_index, segment) in mapfile.segments_list.iter().enumerate() {
        for section in &segment.files_list {
            let mut section_name = section.filepath.to_string_lossy().to_string();
            if let Some(path_decomp_settings) = path_decomp_settings {
                for x in &path_decomp_settings.prefixes_to_trim {
                    if section_name.starts_with(x) {
                        section_name = section_name.trim_start_matches(x).to_string();
                        break;
                    }
                }
                for x in [".s.o", ".c.o", ".cpp.o", ".o"] {
                    if section_name.ends_with(x) {
                        section_name = section_name.trim_end_matches(x).to_string();
                        break;
                    }
                }
            }
            let Some(mut new_report_unit) = report_from_section(section, section_name.clone(), path_decomp_settings) else {
                continue;
            };

            if let Some(report_unit) = units.iter_mut().find(|x| &x.name == &section_name) {
                report_unit.measures =
                    merge_measures(report_unit.measures, new_report_unit.measures);
                report_unit.sections.extend(new_report_unit.sections);
                report_unit.functions.extend(new_report_unit.functions);
            } else {
                let cat = if let Some(x) = section_name.split('/').next() {
                    x
                } else {
                    &segment.name
                }.to_string();

                new_report_unit.metadata = Some(report::ReportUnitMetadata {
                    complete: None,
                    module_name: Some(segment.name.clone()),
                    module_id: Some(segment_index as u32),
                    source_path: None, // mapfile doesn't contain source paths
                    progress_categories: vec![cat.clone()],
                    auto_generated: None,
                });

                progress_categories.insert(cat);

                units.push(new_report_unit);
            }
        }
    }

    for unit in units.iter_mut() {
        if let Some(measures) = &mut unit.measures {
            measures.matched_code_percent = if measures.total_code > 0 {
                measures.matched_code as f32 / measures.total_code as f32 * 100.0
            } else {
                100.0
            };
            measures.matched_data_percent = if measures.total_data > 0 {
                measures.matched_data as f32 / measures.total_data as f32 * 100.0
            } else {
                100.0
            };
            measures.matched_functions_percent =if measures.total_functions > 0 {
                measures.matched_functions as f32 / measures.total_functions as f32 * 100.0
            } else {
                100.0
            };

            let total = measures.total_code + measures.total_data;
            measures.fuzzy_match_percent = if total > 0 {
                (measures.matched_code + measures.matched_data) as f32 / total as f32 * 100.0
            } else {
                100.0
            };
        }
    }

    let mut measures: report::Measures = units.iter().filter_map(|u| u.measures).collect();
    // "the root measures.fuzzy_match_percent is only for code, so I would expect it to be the same as matched_code_percent"
    // - Encounter
    measures.fuzzy_match_percent = measures.matched_code_percent;

    let mut categories = Vec::new();
    for category in progress_categories {
        categories.push(report::ReportCategory {
            id: category.clone(),
            name: category,
            measures: Some(Default::default()),
        });
    }

    let mut report = report::Report {
        measures: Some(measures),
        units,
        version: report::REPORT_VERSION,
        categories,
    };
    report.calculate_progress_categories();

    report
}

lazy_static! {
    static ref BANNED_SECTIONS: HashSet<&'static str> = {
        let mut section_names = HashSet::new();
        section_names.insert(".note");
        section_names.insert(".comment");
        section_names.insert(".pdr");
        section_names.insert(".mdebug");
        section_names.insert(".mdebug.abi32");
        section_names.insert(".debug");
        section_names.insert(".line");
        section_names.insert(".debug_srcinfo");
        section_names.insert(".debug_sfnames");
        section_names.insert(".debug_aranges");
        section_names.insert(".debug_pubnames");
        section_names.insert(".debug_info");
        section_names.insert(".debug_abbrev");
        section_names.insert(".debug_line");
        section_names.insert(".debug_line_end");
        section_names.insert(".debug_frame");
        section_names.insert(".debug_str");
        section_names.insert(".debug_loc");
        section_names.insert(".debug_macinfo");
        section_names.insert(".debug_weaknames");
        section_names.insert(".debug_funcnames");
        section_names.insert(".debug_typenames");
        section_names.insert(".debug_varnames");
        section_names.insert(".debug_pubtypes");
        section_names.insert(".debug_ranges");
        section_names.insert(".debug_addr");
        section_names.insert(".debug_line_str");
        section_names.insert(".debug_loclists");
        section_names.insert(".debug_macro");
        section_names.insert(".debug_names");
        section_names.insert(".debug_rnglists");
        section_names.insert(".debug_str_offsets");
        section_names.insert(".debug_sup");
        section_names.insert(".gnu.attributes");
        section_names
    };
}

fn report_from_section(
    section: &file::File,
    section_name: String,
    path_decomp_settings: Option<&PathDecompSettings>,
) -> Option<report::ReportUnit> {
    if section.is_fill || BANNED_SECTIONS.contains(section.section_type.as_str()) {
        return None;
    }

    let mut measures = report::Measures::default();
    let mut report_item = report_item_from_section(section);
    let mut functions = Vec::new();

    let is_text = section.section_type.starts_with(".text") | section.section_type.starts_with(".start") | section.section_type.starts_with(".init");
    let track_data = false;

    for (i, sym_state) in section.symbol_match_state_iter(path_decomp_settings).enumerate() {
        let mut fuzzy_match_percent = 0.0;

        let sym = match sym_state {
            file::SymbolDecompState::Decomped(sym) => {
                let static_size = sym.vram - section.vram;

                if is_text {
                    measures.matched_code += sym.size;
                    measures.matched_functions += 1;
                    fuzzy_match_percent = 100.0;

                    if i == 0 && sym.vram != section.vram {
                        measures.matched_code += static_size;
                        measures.matched_functions += 1;
                        fuzzy_match_percent = 100.0;
                    }
                } else {
                    measures.matched_data += if track_data {
                        sym.size
                    } else {
                        0
                    };

                    if i == 0 && sym.vram != section.vram {
                        measures.matched_data += if track_data {
                            static_size
                        } else {
                            0
                        };
                    }
                }
                sym.clone()
            }
            file::SymbolDecompState::Undecomped(sym) => sym,
        };

        if is_text {
            measures.total_code += sym.size;
            measures.total_functions += 1;

            functions.push(report::ReportItem {
                name: sym.name.clone(),
                size: sym.size,
                fuzzy_match_percent,
                metadata: Some(report::ReportItemMetadata {
                    demangled_name: None,
                    virtual_address: Some(sym.vram),
                }),
                address: Some(sym.vram - section.vram),
            });

            if i == 0 && sym.vram != section.vram {
                // First symbol is a static symbol, so fake a placeholder
                let static_vram = section.vram;
                let static_size = sym.vram - section.vram;

                measures.total_code += static_size;
                measures.total_functions += 1;

                functions.push(report::ReportItem {
                    name: format!("$_static_symbol_{:08X}_{}", static_vram, section.filepath.display()),
                    size: static_size,
                    fuzzy_match_percent,
                    metadata: Some(report::ReportItemMetadata {
                        demangled_name: None,
                        virtual_address: Some(static_vram),
                    }),
                    address: Some(0),
                });
            }
        } else {
            measures.total_data += if track_data {
                sym.size
            } else {
                0
            };

            if i == 0 && sym.vram != section.vram {
                // First symbol is a static symbol, so fake a placeholder
                let static_size = sym.vram - section.vram;
                measures.total_data += if track_data {
                    static_size
                } else {
                    0
                };
            }
        }
    }

    if measures.total_code + measures.total_data > 0 {
        report_item.fuzzy_match_percent = (measures.matched_code + measures.matched_data) as f32
            / (measures.total_code + measures.total_data) as f32
            * 100.0;
    }

    // An unit always contains a singular unit, no more, no less. Right?
    measures.total_units = 1;

    Some(report::ReportUnit {
        name: section_name,
        measures: Some(measures),
        sections: vec![report_item],
        functions,
        metadata: None,
    })
}

fn merge_measures(
    a: Option<report::Measures>,
    b: Option<report::Measures>,
) -> Option<report::Measures> {
    match (a, b) {
        (None, None) => None,
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        (Some(a), Some(b)) => Some(report::Measures {
            fuzzy_match_percent: 0.0,
            total_code: a.total_code + b.total_code,
            matched_code: a.matched_code + b.matched_code,
            matched_code_percent: 0.0,
            total_data: a.total_data + b.total_data,
            matched_data: a.matched_data + b.matched_data,
            matched_data_percent: 0.0,
            total_functions: a.total_functions + b.total_functions,
            matched_functions: a.matched_functions + b.matched_functions,
            matched_functions_percent: 0.0,
            complete_code: 0,
            complete_code_percent: 0.0,
            complete_data: 0,
            complete_data_percent: 0.0,
            total_units: 1,
            complete_units: 0,
        }),
    }
}

fn report_item_from_section(section: &file::File) -> report::ReportItem {
    report::ReportItem {
        name: section.section_type.clone(),
        size: section.size,
        fuzzy_match_percent: 0.0,
        metadata: Some(report::ReportItemMetadata {
            demangled_name: None,
            virtual_address: Some(section.vram),
        }),
        address: None,
    }
}
