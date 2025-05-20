#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2025 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import TextIO
import sys

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

    if decompConfig is not None:
        version = decompConfig.get_version_by_name(args.version)
        mapPath = Path(args.mapfile if args.mapfile is not None else version.paths.get("map"))
        asmPath = Path(args.asmpath if args.asmpath is not None else version.paths.get("asm"))

        prefixesToTrim = list(args.prefixes_to_trim)
        if len(prefixesToTrim) > 0:
            pass
    else:
        mapPath = args.mapfile
        asmPath = args.asmpath
        prefixesToTrim = args.prefixes_to_trim

    outputPath: Path = args.output
    pathIndex: int = args.path_index

    exit(doObjdiffReport(mapPath, asmPath, outputPath, prefixesToTrim, reportCategories, pathIndex=pathIndex))

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser], decompConfig=None):
    parser = subparser.add_parser("objdiff_report", help="Computes current progress of the matched functions. Expects `.NON_MATCHING` marker symbols on the mapfile to know which symbols are not matched yet.")

    nargs: str|int = 1
    if decompConfig is not None:
        nargs = "?"
        versions = []
        for version in decompConfig.versions:
            versions.append(version.name)
        parser.add_argument("-v", "--version", help="Version to process from the decomp.yaml file", type=str, choices=versions, default=versions[0])

    parser.add_argument("mapfile", help="Path to a map file. This argument is optional if an `decomp.yaml` file is detected on the current project.", type=Path, nargs=nargs)
    parser.add_argument("asmpath", help="Path to asm folder. This argument is optional if an `decomp.yaml` file is detected on the current project.", type=Path, nargs=nargs)
    parser.add_argument("output", help="Path to output file.", type=Path)
    parser.add_argument("-t", "--prefixes-to-trim", help="", action="append")
    parser.add_argument("-i", "--path-index", help="Specify the index to start reading the file paths. Defaults to 2", type=int, default=2)

    parser.set_defaults(func=processArguments)
