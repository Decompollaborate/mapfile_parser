#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2023-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

## This test checks if it is possible to modify a class's member
## (important because of Rust/Python interoperability)

import mapfile_parser
from pathlib import Path

def getProgressFromMapFile(mapFile: mapfile_parser.MapFile, aliases: dict[str, str]=dict(), pathIndex: int=2) -> tuple[mapfile_parser.ProgressStats, dict[str, mapfile_parser.ProgressStats]]:
    totalStats = mapfile_parser.ProgressStats()
    progressPerFolder: dict[str, mapfile_parser.ProgressStats] = dict()

    for segment in mapFile:
        for file in segment:
            if len(file) == 0:
                continue

            folder = file.filepath.parts[pathIndex]
            if folder in aliases:
                folder = aliases[folder]

            if folder not in progressPerFolder:
                progressPerFolder[folder] = mapfile_parser.ProgressStats()

            for func in file:
                if func.name.endswith(".NON_MATCHING"):
                    continue

                funcNonMatching = f"{func.name}.NON_MATCHING"

                funcSize = func.size
                assert funcSize is not None

                if mapFile.findSymbolByName(funcNonMatching) is not None:
                    totalStats.undecompedSize += funcSize
                    progressPerFolder[folder].undecompedSize += funcSize
                else:
                    totalStats.decompedSize += funcSize
                    progressPerFolder[folder].decompedSize += funcSize

    return totalStats, progressPerFolder

def getProgress(mapPath: Path, version: str, pathIndex: int=2) -> tuple[mapfile_parser.ProgressStats, dict[str, mapfile_parser.ProgressStats]]:
    mapFile = mapfile_parser.MapFile()
    mapFile.readMapFile(mapPath)

    for segment in mapFile:
        for file in segment:
            if len(file) == 0:
                continue

            filepathParts = list(file.filepath.parts)
            if version in filepathParts:
                filepathParts.remove(version)
            file.filepath = Path(*filepathParts)

            # Fix symbol size calculation because of NON_MATCHING symbols
            for sym in file:
                if sym.name.endswith(".NON_MATCHING") and sym.size != 0:
                    realSym = file.findSymbolByName(sym.name.replace(".NON_MATCHING", ""))
                    if realSym is not None and realSym.size == 0:
                        realSym.size = sym.size
                        sym.size = 0

    return getProgressFromMapFile(mapFile.filterBySectionType(".text"), aliases={"ultralib": "libultra"}, pathIndex=pathIndex)


cases: list[tuple[Path, str, mapfile_parser.ProgressStats]] = [
    (Path("tests/maps/drmario64.cn.map"),       "cn",  mapfile_parser.ProgressStats(undecompedSize=273028, decompedSize=199196)),
    (Path("tests/maps/drmario64.us.lld.map"),   "us",  mapfile_parser.ProgressStats(undecompedSize=170720, decompedSize=272764)),
    (Path("tests/maps/drmario64.us.map"),       "us",  mapfile_parser.ProgressStats(undecompedSize=170720, decompedSize=272128)),
    (Path("tests/maps/puzzleleague64.usa.map"), "usa", mapfile_parser.ProgressStats(undecompedSize=263668, decompedSize=454604)),
]


errors = 0
for (mapPath, version, expected) in cases:
    print(mapPath)

    totalStats, progressPerFolder = getProgress(mapPath, version)

    print(f"    {expected} {expected.decompedPercentage():>10.4f}%")
    print(f"    {totalStats} {totalStats.decompedPercentage():>10.4f}%")

    if totalStats == expected:
        print("        Ok")
    else:
        print("        Wrong")
        errors += 1

    print()

print(f"Total errors: {errors}")
exit(errors)
