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
    let path_index = if let Some(path_decomp_settings) = path_decomp_settings {
        path_decomp_settings.path_index
    } else {
        0
    };

    for (segment_index, segment) in mapfile.segments_list.iter().enumerate() {
        for section in &segment.files_list {
            let section_path = section.filepath.to_string_lossy().to_string();
            let mut new_report_unit = report_from_section(section, path_decomp_settings);

            if let Some(report_unit) = units.iter_mut().find(|x| x.name == section_path) {
                report_unit.measures =
                    merge_measures(report_unit.measures, new_report_unit.measures);
                report_unit.sections.extend(new_report_unit.sections);
                report_unit.functions.extend(new_report_unit.functions);
            } else {
                let cat = match section.filepath.components().nth(path_index) {
                    Some(x) if path_index > 0 => x,
                    _ => section
                        .filepath
                        .components()
                        .nth(section.filepath.components().count().saturating_sub(1))
                        .unwrap(),
                }
                .as_os_str()
                .to_str()
                .unwrap()
                .to_string();

                new_report_unit.metadata = Some(report::ReportUnitMetadata {
                    complete: None,
                    module_name: Some(segment.name.clone()),
                    module_id: Some(segment_index as u32),
                    source_path: Some(section_path),
                    progress_categories: vec![cat.clone()],
                    auto_generated: None, // TODO: What?
                });

                progress_categories.insert(cat);

                units.push(new_report_unit);
            }
        }
    }

    for unit in units.iter_mut() {
        if let Some(measures) = &mut unit.measures {
            if measures.total_code > 0 {
                measures.matched_code_percent =
                    measures.matched_code as f32 / measures.total_code as f32 * 100.0;
            }
            if measures.total_data > 0 {
                measures.matched_data_percent =
                    measures.matched_data as f32 / measures.total_data as f32 * 100.0;
            }
            if measures.total_functions > 0 {
                measures.matched_functions_percent =
                    measures.matched_functions as f32 / measures.total_functions as f32 * 100.0;
            }

            let total = measures.total_code + measures.total_data;
            if total > 0 {
                measures.fuzzy_match_percent =
                    (measures.matched_code + measures.matched_data) as f32 / total as f32 * 100.0;
            }
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

fn report_from_section(
    section: &file::File,
    path_decomp_settings: Option<&PathDecompSettings>,
) -> report::ReportUnit {
    let mut measures = report::Measures::default();
    let mut report_item = report_item_from_section(section);
    let mut functions = Vec::new();

    let is_text = matches!(section.section_type.as_str(), ".text" | ".start");

    for sym_state in section.symbol_match_state_iter(path_decomp_settings) {
        let mut fuzzy_match_percent = 0.0;

        let sym = match sym_state {
            file::SymbolDecompState::Decomped(sym) => {
                if is_text {
                    measures.matched_code += sym.size;
                    measures.matched_functions += 1;
                    fuzzy_match_percent = 100.0;
                } else {
                    measures.matched_data += sym.size;
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
                address: Some(sym.vram),
            });
        } else {
            measures.total_data += sym.size;
        }
    }

    if measures.total_code + measures.total_data > 0 {
        report_item.fuzzy_match_percent = (measures.matched_code + measures.matched_data) as f32
            / (measures.total_code + measures.total_data) as f32
            * 100.0;
    }

    // An unit always contains a singular unit, no more, no less. Right?
    measures.total_units = 1;

    report::ReportUnit {
        name: section.filepath.to_string_lossy().to_string(),
        measures: Some(measures),
        sections: vec![report_item],
        functions,
        metadata: None,
    }
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
        address: Some(section.vram)
    }
}
