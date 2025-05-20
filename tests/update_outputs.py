#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2023-2025 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import mapfile_parser
from pathlib import Path

print(f"Running mapfile_parser version {mapfile_parser.__version__}")

outputFolder = Path("tests/output/")
outputFolder.mkdir(parents=True, exist_ok=True)

versions = ["us", "cn", "gw", "usa"]
patterns = [
    "build/{version}/lib/",
    "build/{version}/src/",
    "build/{version}/asm/data",
    "build/{version}/asm/",
    "build/{version}/",
]
prefixesToTrim = []
for v in versions:
    prefixesToTrim += [pattern.format(version=v) for pattern in patterns]
prefixesToTrim.append("build/")

reportCategories = mapfile_parser.ReportCategories()

for mapPath in sorted(Path("tests/maps").iterdir()):
    print(mapPath)

    print("    .json")
    mapfile_parser.frontends.jsonify.doJsonify(mapPath, outputFolder/mapPath.with_suffix(".json").name)

    print("    .machine.json")
    mapfile_parser.frontends.jsonify.doJsonify(mapPath, outputFolder/mapPath.with_suffix(".machine.json").name, humanReadable=False)

    print("    .sym")
    mapfile_parser.frontends.pj64_syms.doPj64Syms(mapPath, outputFolder/mapPath.with_suffix(".sym").name)

    print("    .objdiff_report.json")
    mapfile_parser.frontends.objdiff_report.doObjdiffReport(
        mapPath,
        Path(),
        outputFolder/mapPath.with_suffix(".objdiff_report.json").name,
        prefixesToTrim,
        reportCategories,
    )

    print("    .csv")
    mapfile_parser.frontends.symbol_sizes_csv.doSymbolSizesCsv(mapPath, outputFolder/mapPath.with_suffix(".csv").name)

    print("    .symbols.csv")
    mapfile_parser.frontends.symbol_sizes_csv.doSymbolSizesCsv(mapPath, outputFolder/mapPath.with_suffix(".symbols.csv").name, symbolsSummary=True)
