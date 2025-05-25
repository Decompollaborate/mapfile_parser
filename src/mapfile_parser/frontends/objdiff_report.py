#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2025 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import dataclasses
import decomp_settings
from pathlib import Path

from .. import mapfile


def doObjdiffReport(mapPath: Path, outputPath: Path, prefixesToTrim: list[str], reportCategories: mapfile.ReportCategories, *, pathIndex: int=2, asmPath: Path|None=None, nonmatchingsPath: Path|None=None) -> int:
    if not mapPath.exists():
        print(f"Could not find mapfile at '{mapPath}'")
        return 1

    mapFile = mapfile.MapFile()
    mapFile.readMapFile(mapPath)

    mapFile.writeObjdiffReportToFile(
        outputPath,
        prefixesToTrim,
        reportCategories,
        pathIndex=pathIndex,
        asmPath=asmPath,
        nonmatchingsPath=nonmatchingsPath,
    )

    return 0

def processArguments(args: argparse.Namespace, decompConfig: decomp_settings.Config|None=None):
    reportCategories = mapfile.ReportCategories()
    outputPath: Path
    pathIndexDefault = 2

    settings = SpecificSettings.fromDecompConfig(decompConfig)
    if settings is not None:
        if settings.output is not None:
            outputPath = settings.output
        else:
            outputPath = args.output

        if len(settings.prefixesToTrim) > 0:
            prefixesToTrim = settings.prefixesToTrim
        elif args.prefixes_to_trim is not None:
            prefixesToTrim = list(args.prefixes_to_trim)
        else:
            prefixesToTrim = []

        for cat in settings.categories:
            reportCategories.push(cat.ide, cat.name, cat.paths)

        if settings.pathIndex is not None:
            pathIndex = settings.pathIndex
        elif args.path_index is not None:
            pathIndex = int(args.path_index)
        else:
            pathIndex = pathIndexDefault
    else:
        outputPath = args.output
        if args.prefixes_to_trim is not None:
            prefixesToTrim = list(args.prefixes_to_trim)
        else:
            prefixesToTrim = []
        pathIndex = int(args.path_index) if args.path_index is not None else pathIndexDefault

    if decompConfig is not None:
        version = decompConfig.get_version_by_name(args.version)
        assert version is not None
        mapPath = Path(version.paths.map)

        if version.paths.asm is None:
            asmPath = args.asmpath
        elif settings is not None and settings.checkAsmPaths:
            asmPath = Path(version.paths.asm)
        else:
            asmPath = None

        if version.paths.nonmatchings is None:
            nonmatchingsPath = args.nonmatchingspath
        elif settings is not None and settings.checkAsmPaths:
            nonmatchingsPath = Path(version.paths.nonmatchings)
        else:
            nonmatchingsPath = None
    else:
        mapPath = args.mapfile
        asmPath = args.asmpath
        nonmatchingsPath = args.nonmatchingspath

    exit(doObjdiffReport(mapPath, outputPath, prefixesToTrim, reportCategories, asmPath=asmPath, pathIndex=pathIndex, nonmatchingsPath=nonmatchingsPath))

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser], decompConfig: decomp_settings.Config|None=None):
    epilog = """\
Visit https://decomp.dev/ and https://wiki.decomp.dev/tools/decomp-dev for more
information about uploading the generated progress report.

This utility has support for a special section on the `decomp.yaml` file, which
allows to avoid passing many arguments to utility.

Here's an example for this entry:

```
tools:
  mapfile_parser:
    progress_report:
      # output: report.json # Optional
      check_asm_paths: True
      # path_index: 2
      # List of build prefixes to trim from each object file
      prefixes_to_trim:
        - build/lib/
        - build/src/
        - build/asm/data/
        - build/asm/
        - build/
      # List of categories. `id`s must be unique, but each path may be
      # duplicated across categories.
      categories:
        - id: rom_header
          name: rom_header
          paths:
            - rom_header/

        - id: game_cod
          name: Game code
          paths:
            - main_segment/

        - id: libultra
          name: libultra
          paths:
            - libultra/
            - ultralib/
```
"""

    parser = subparser.add_parser(
        "objdiff_report",
        help="Computes current progress of the matched functions and generates a report suitable for uploading to https://decomp.dev/. Expects `.NON_MATCHING` marker symbols on the mapfile to know which symbols are not matched yet.",
        epilog=epilog,
        formatter_class=argparse.RawTextHelpFormatter,
    )

    emitMapfile = True
    emitOutput = True
    emitAsmpath = True
    emitNonmatchings = True
    emitPrefixesToTrim = True
    emitPathIndex = True
    settings = SpecificSettings.fromDecompConfig(decompConfig)
    if settings is not None:
        assert decompConfig is not None
        versions = []
        for version in decompConfig.versions:
            versions.append(version.name)

        if len(versions) > 0:
            parser.add_argument("-v", "--version", help="Version to process from the decomp.yaml file", type=str, choices=versions, default=versions[0])
            emitMapfile = False
            if decompConfig.versions[0].paths.asm is not None:
                emitAsmpath = False
            if decompConfig.versions[0].paths.nonmatchings is not None:
                emitNonmatchings = False
        if settings.output is not None:
            emitOutput = False
        if len(settings.prefixesToTrim) > 0:
            emitPrefixesToTrim = False
        if settings.pathIndex is not None:
            emitPathIndex = False

    # CLI options exists only if they are not present on the decomp.yaml file
    if emitMapfile:
        parser.add_argument("mapfile", help="Path to a map file.", type=Path)
    if emitOutput:
        parser.add_argument("output", help="Path to output file.", type=Path)

    if emitAsmpath:
        parser.add_argument("-a", "--asmpath", help="Path to asm folder.", type=Path)
    if emitNonmatchings:
        parser.add_argument("-n", "--nonmatchingspath", help="Path to nonmatchings folder.", type=Path)
    if emitPrefixesToTrim:
        parser.add_argument("-t", "--prefixes-to-trim", help="List of path prefixes to try to trim from each object path from the mapfile. For each object they will be tried in order and it will stop at the first prefix found.", action="append")
    if emitPathIndex:
        parser.add_argument("-i", "--path-index", help="Specify the index to start reading the file paths. Defaults to 2", type=int)

    parser.set_defaults(func=processArguments)


@dataclasses.dataclass
class SpecificSettings:
    output: Path|None
    prefixesToTrim: list[str]
    categories: list[Category]
    pathIndex: int|None
    checkAsmPaths: bool

    @staticmethod
    def fromDecompConfig(decompConfig) -> SpecificSettings|None:
        if decompConfig is None:
            return None

        output: Path|None = None
        prefixesToTrim: list[str] = []
        categories: list[Category] = []
        pathIndex: int|None = None
        checkAsmPaths: bool = False
        if decompConfig.tools is not None:
            mapfileParserConfig = decompConfig.tools.get("mapfile_parser")
            if mapfileParserConfig is not None:
                raw = mapfileParserConfig.raw()
                if isinstance(raw, dict):
                    raw = raw.get("progress_report")
                    if isinstance(raw, dict):
                        var = raw.get("output")
                        if var is not None:
                            output = Path(var)
                        var = raw.get("prefixes_to_trim")
                        if var is not None:
                            prefixesToTrim = list(var)
                        var = raw.get("categories")
                        if var is not None:
                            for x in var:
                                cat = Category.from_dict(x)
                                assert cat is not None, f"Invalid category {x}"
                                categories.append(cat)
                        var = raw.get("path_index")
                        if var is not None:
                            pathIndex = var
                        checkAsmPaths = bool(raw.get("check_asm_paths", False))

        return SpecificSettings(
            output,
            prefixesToTrim,
            categories,
            pathIndex,
            checkAsmPaths,
        )

@dataclasses.dataclass
class Category:
    ide: str
    name: str
    paths: list[str]

    @staticmethod
    def from_dict(data: dict|None) -> Category|None:
        if data is None:
            return None

        ide = data.get("id")
        if ide is None:
            return None
        assert isinstance(ide, str)

        name = data.get("name")
        if name is None:
            return None
        assert isinstance(name, str)

        paths = data.get("paths")
        if paths is None:
            return None
        assert isinstance(paths, list)
        for x in paths:
            assert isinstance(x, str)

        return Category(
            ide,
            name,
            paths,
        )
