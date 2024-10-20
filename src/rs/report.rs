/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use objdiff_core::bindings::report;
use crate::{mapfile, file};

impl mapfile::MapFile {
    pub fn get_objdiff_report(&self) -> report::Report {
        do_report(self)
    }
}

fn do_report(mapfile: &mapfile::MapFile) -> report::Report {
    let mut units: Vec<report::ReportUnit> = Vec::new();

    for (segment_index, segment) in mapfile.segments_list.iter().enumerate() {
        for section in &segment.files_list {
            let section_path = section.filepath.to_string_lossy().to_string();

            if let Some(report_unit) = units.iter_mut().find(|x| x.name == section_path) {
                report_unit.measures = measures_from_section(report_unit.measures, section);
                report_unit.sections.push(report_item_from_section(section));
                report_unit.functions.extend(gather_functions_from_section(section));
            } else {
                let report_unit = report::ReportUnit {
                    name: section_path.clone(),
                    measures: measures_from_section(None, section),
                    sections: vec![report_item_from_section(section)],
                    functions: gather_functions_from_section(section),
                    metadata: Some(report::ReportUnitMetadata {
                        complete: None, // TODO
                        module_name: Some(segment.name.clone()),
                        module_id: Some(segment_index as u32),
                        source_path: Some(section_path),
                        progress_categories: Vec::new(), // TODO
                        auto_generated: None, // TODO: What?
                    }),
                };

                units.push(report_unit);
            }
        }
    }

    let measures = units.iter().flat_map(|u| u.measures.into_iter()).collect();

    let categories = Vec::new();
    // TODO: fill categories


    let mut report = report::Report { measures: Some(measures), units: units, version: report::REPORT_VERSION, categories: categories };
    report.calculate_progress_categories();

    report
}

fn measures_from_section(measures_aux: Option<report::Measures>, section: &file::File) -> Option<report::Measures> {
    if section.size == 0 {
        return None;
    }

    let measures = measures_aux.unwrap_or_default();
    // TODO
    Some(measures)
}

fn report_item_from_section(section: &file::File) -> report::ReportItem {
    report::ReportItem {
        name: format!("{:?}({})", section.filepath, section.section_type),
        size: section.size,
        fuzzy_match_percent: 0.0, // TODO
        metadata: Some(report::ReportItemMetadata {
            demangled_name: None,
            virtual_address: Some(section.vram),
        }),
    }
}

fn gather_functions_from_section(section: &file::File) -> Vec<report::ReportItem> {
    if section.section_type != ".text" && section.section_type != ".start" {
        return Vec::new()
    }

    let mut funcs = Vec::new();

    for sym in &section.symbols {
        funcs.push(report::ReportItem {
            name: sym.name.clone(),
            size: sym.size.unwrap_or(0),
            fuzzy_match_percent: 0.0, // TODO
            metadata: Some(report::ReportItemMetadata {
                demangled_name: None,
                virtual_address: Some(sym.vram)
            }),
        });
    }

    funcs
}
