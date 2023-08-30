#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2023 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import mapfile_parser
from pathlib import Path

print(f"Running mapfile_parser version {mapfile_parser.__version__}")

outputFolder = Path("tests/output/")
outputFolder.mkdir(parents=True, exist_ok=True)

for mapPath in sorted(Path("tests/maps").iterdir()):
    print(mapPath)

    print("    .json")
    mapfile_parser.frontends.jsonify.doJsonify(mapPath, outputFolder/mapPath.with_suffix(".json").name)

    print("    .machine.json")
    mapfile_parser.frontends.jsonify.doJsonify(mapPath, outputFolder/mapPath.with_suffix(".machine.json").name, humanReadable=False)

    print("    .sym")
    mapfile_parser.frontends.pj64_syms.doPj64Syms(mapPath, outputFolder/mapPath.with_suffix(".sym").name)

    print("    .csv")
    mapfile_parser.frontends.symbol_sizes_csv.doSymbolSizesCsv(mapPath, outputFolder/mapPath.with_suffix(".csv").name)

    print("    .symbols.csv")
    mapfile_parser.frontends.symbol_sizes_csv.doSymbolSizesCsv(mapPath, outputFolder/mapPath.with_suffix(".symbols.csv").name, symbolsSummary=True)
