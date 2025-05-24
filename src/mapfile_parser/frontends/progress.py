#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import decomp_settings
import json
from pathlib import Path

from .. import mapfile
from .. import progress_stats


def getProgress(mapPath: Path, asmPath: Path, nonmatchingsPath: Path, pathIndex: int=2, checkFunctionFiles: bool=True, debugging: bool=False) -> tuple[progress_stats.ProgressStats, dict[str, progress_stats.ProgressStats]]:
    mapFile = mapfile.MapFile()

    mapFile.debugging = debugging
    mapFile.readMapFile(mapPath)

    return mapFile.filterBySectionType(".text").getProgress(asmPath, nonmatchingsPath, pathIndex=pathIndex, checkFunctionFiles=checkFunctionFiles)

def doProgress(mapPath: Path, asmPath: Path, nonmatchingsPath: Path, pathIndex: int=2, checkFunctionFiles: bool=True, print_json: bool=False, debugging: bool=False) -> int:
    if not mapPath.exists():
        print(f"Could not find mapfile at '{mapPath}'")
        return 1

    totalStats, progressPerFolder = getProgress(mapPath, asmPath, nonmatchingsPath, pathIndex=pathIndex, checkFunctionFiles=checkFunctionFiles, debugging=debugging)

    if print_json:
        json_temp: dict[str, dict[str, int|float]] = {
            "all": totalStats.asJsonEntry()
        }
        for folder, statsEntry in progressPerFolder.items():
            json_temp[folder] = statsEntry.asJsonEntry()
        print(json.dumps(json_temp, indent=4))
    else:
        progress_stats.printStats(totalStats, progressPerFolder)
    return 0


def processArguments(args: argparse.Namespace, decompConfig: decomp_settings.Config|None=None):
    if decompConfig is not None:
        version = decompConfig.get_version_by_name(args.version)
        assert version is not None, f"Invalid version '{args.version}' selected"

        mapPath = Path(version.paths.map)
        asmPath = Path(version.paths.asm if version.paths.asm is not None else args.asmpath)
        nonmatchingsPath = Path(version.paths.nonmatchings if version.paths.nonmatchings is not None else args.nonmatchingspath)
    else:
        mapPath = args.mapfile
        asmPath = args.asmpath
        nonmatchingsPath = args.nonmatchingspath

    pathIndex: int = args.path_index
    checkFunctionFiles: bool = not args.avoid_function_files
    debugging: bool = args.debugging #! @deprecated
    print_json: bool = args.json

    exit(doProgress(mapPath, asmPath, nonmatchingsPath, pathIndex=pathIndex, checkFunctionFiles=checkFunctionFiles, print_json=print_json, debugging=debugging))

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser], decompConfig: decomp_settings.Config|None=None):
    parser = subparser.add_parser("progress", help="Computes current progress of the matched functions. Relies on a splat (https://github.com/ethteck/splat) folder structure and matched functions not longer having a file.")

    emitMapfile = True
    emitAsmpath = True
    emitNonmatchingsPath = True
    if decompConfig is not None:
        versions = []
        for version in decompConfig.versions:
            versions.append(version.name)

        if len(versions) > 0:
            parser.add_argument("-v", "--version", help="Version to process from the decomp.yaml file", type=str, choices=versions, default=versions[0])
            emitMapfile = False
            if decompConfig.versions[0].paths.asm is not None:
                emitAsmpath = False
            if decompConfig.versions[0].paths.nonmatchings is not None:
                emitNonmatchingsPath = False

    if emitMapfile:
        parser.add_argument("mapfile", help="Path to a map file.", type=Path)
    if emitAsmpath:
        parser.add_argument("asmpath", help="Path to asm folder.", type=Path)
    if emitNonmatchingsPath:
        parser.add_argument("nonmatchingspath", help="Path to nonmatchings folder.", type=Path)

    parser.add_argument("-i", "--path-index", help="Specify the index to start reading the file paths. Defaults to 2", type=int, default=2)
    parser.add_argument("-f", "--avoid-function-files", help="Avoid checking if the assembly file for a function exists as a way to determine if the function has been matched or not", action="store_true")
    parser.add_argument("-d", "--debugging", help="Enable debugging prints. This option is deprecated", action="store_true")
    parser.add_argument("-j", "--json", help="Print the stats as json instead of a human readable format.", action="store_true")

    parser.set_defaults(func=processArguments)
