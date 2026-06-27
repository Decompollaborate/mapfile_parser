#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2023-2025 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import mapfile_parser
from pathlib import Path
import shutil

print(f"Running mapfile_parser version {mapfile_parser.__version__}")

mapsPath = Path("tests/maps")

outputFolder = Path("tests/output/")
shutil.rmtree(outputFolder)
outputFolder.mkdir(parents=True, exist_ok=True)

versions = ["us", "cn", "gw", "usa"]
patterns = [
    "build/{version}/lib/",
    "build/{version}/src/",
    "build/{version}/asm/data",
    "build/{version}/asm/",
    "build/{version}/",
]
prefixesToTrim: list[str] = []
for v in versions:
    prefixesToTrim += [pattern.format(version=v) for pattern in patterns]
prefixesToTrim.append("build/")

reportCategories = mapfile_parser.ReportCategories()
reportCategories.setReportData(True)


def plfResolver(x: Path) -> Path | None:
    if x.suffix == ".plf":
        newPath = x.with_suffix(".map")
        if newPath.exists():
            return newPath
    return None


for mapPath in sorted(mapsPath.rglob("*")):
    if not mapPath.is_file():
        continue

    print(mapPath)

    print("    .json")
    mapfile_parser.frontends.jsonify.doJsonify(
        mapPath,
        outputFolder / mapPath.with_suffix(".json").relative_to(mapsPath),
        plfResolver=plfResolver,
    )

    print("    .machine.json")
    mapfile_parser.frontends.jsonify.doJsonify(
        mapPath,
        outputFolder / mapPath.with_suffix(".machine.json").relative_to(mapsPath),
        humanReadable=False,
        plfResolver=plfResolver,
    )

    print("    .sym")
    mapfile_parser.frontends.pj64_syms.doPj64Syms(
        mapPath,
        outputFolder / mapPath.with_suffix(".sym").relative_to(mapsPath),
        plfResolver=plfResolver,
    )

    print("    .objdiff_report.json")
    mapfile_parser.frontends.objdiff_report.doObjdiffReport(
        mapPath,
        outputFolder
        / mapPath.with_suffix(".objdiff_report.json").relative_to(mapsPath),
        prefixesToTrim,
        reportCategories,
        quiet=True,
        plfResolver=plfResolver,
    )

    print("    .csv")
    mapfile_parser.frontends.symbol_sizes_csv.doSymbolSizesCsv(
        mapPath,
        outputFolder / mapPath.with_suffix(".csv").relative_to(mapsPath),
        plfResolver=plfResolver,
    )

    print("    .symbols.csv")
    mapfile_parser.frontends.symbol_sizes_csv.doSymbolSizesCsv(
        mapPath,
        outputFolder / mapPath.with_suffix(".symbols.csv").relative_to(mapsPath),
        symbolsSummary=True,
        plfResolver=plfResolver,
    )
