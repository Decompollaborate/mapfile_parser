#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2025 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import dataclasses
from pathlib import Path

from .. import mapfile


def doObjdiffReport(mapPath: Path, asmPath: Path, outputPath: Path, prefixesToTrim: list[str], reportCategories: mapfile.ReportCategories, *, pathIndex: int=2) -> int:
    if not mapPath.exists():
        print(f"Could not find mapfile at '{mapPath}'")
        return 1

    mapFile = mapfile.MapFile()
    mapFile.readMapFile(mapPath)

    mapFile.writeObjdiffReportToFile(
        outputPath,
        prefixesToTrim,
        reportCategories,
        asmPath,
        pathIndex=pathIndex,
    )

    return 0

def processArguments(args: argparse.Namespace, decompConfig=None):
    reportCategories = mapfile.ReportCategories()
    pathIndexDefault = 2

    if decompConfig is not None:
        version = decompConfig.get_version_by_name(args.version)
        assert version is not None
        mapPath = Path(args.mapfile if args.mapfile is not None else version.paths["map"])
        asmPath = Path(args.asmpath if args.asmpath is not None else version.paths["asm"])

    else:
        mapPath = args.mapfile
        asmPath = args.asmpath

    settings = SpecificSettings.fromDecompConfig(decompConfig)
    if settings is not None:
        if args.output is not None:
            outputPath = args.output
        else:
            assert settings.output is not None
            outputPath = settings.output

        if args.prefixes_to_trim is not None:
            prefixesToTrim = list(args.prefixes_to_trim)
        else:
            prefixesToTrim = settings.prefixesToTrim

        for cat in settings.categories:
            reportCategories.push(cat.ide, cat.name, cat.paths)

        if args.path_index is not None:
            pathIndex = int(args.path_index)
        elif settings.pathId is not None:
            pathIndex = settings.pathId
        else:
            pathIndex = pathIndexDefault
    else:
        outputPath: Path = args.output
        prefixesToTrim = args.prefixes_to_trim
        pathIndex = int(args.path_index) if args.path_index is not None else pathIndexDefault

    exit(doObjdiffReport(mapPath, asmPath, outputPath, prefixesToTrim, reportCategories, pathIndex=pathIndex))

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser], decompConfig=None):
    parser = subparser.add_parser("objdiff_report", help="Computes current progress of the matched functions. Expects `.NON_MATCHING` marker symbols on the mapfile to know which symbols are not matched yet.")

    nargs_mapfile: str|int = 1
    nargs_asmpath: str|int = 1
    nargs_output: str|int = 1
    settings = SpecificSettings.fromDecompConfig(decompConfig)
    if settings is not None:
        assert decompConfig is not None
        versions = []
        for version in decompConfig.versions:
            versions.append(version.name)

        if len(versions) > 0:
            parser.add_argument("-v", "--version", help="Version to process from the decomp.yaml file", type=str, choices=versions, default=versions[0])
            nargs_mapfile = "?"
            nargs_asmpath = "?"
        if settings.output is not None:
            nargs_output = "?"

    parser.add_argument("mapfile", help="Path to a map file. This argument is optional if an `decomp.yaml` file is detected on the current project.", type=Path, nargs=nargs_mapfile)
    parser.add_argument("asmpath", help="Path to asm folder. This argument is optional if an `decomp.yaml` file is detected on the current project.", type=Path, nargs=nargs_asmpath)
    parser.add_argument("output", help="Path to output file.", type=Path, nargs=nargs_output)
    parser.add_argument("-t", "--prefixes-to-trim", help="", action="append")
    parser.add_argument("-i", "--path-index", help="Specify the index to start reading the file paths. Defaults to 2", type=int)

    parser.set_defaults(func=processArguments)


@dataclasses.dataclass
class SpecificSettings:
    output: Path|None
    prefixesToTrim: list[str]
    categories: list[Category]
    pathId: int|None

    @staticmethod
    def fromDecompConfig(decompConfig) -> SpecificSettings|None:
        if decompConfig is None:
            return None

        output = None
        prefixesToTrim = []
        categories = []
        pathId = None
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
                                categories.append(Category.from_dict(x))
                        var = raw.get("path_index")
                        if var is not None:
                            pathId = var

        return SpecificSettings(
            output,
            prefixesToTrim,
            categories,
            pathId,
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
