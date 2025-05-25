#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2023-2025 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import decomp_settings
import json
from pathlib import Path

from .. import mapfile


def doJsonify(mapPath: Path, outputPath: Path|None, humanReadable: bool=True, applyFixes: bool=False) -> int:
    if not mapPath.exists():
        print(f"Could not find mapfile at '{mapPath}'")
        return 1

    mapFile = mapfile.MapFile()
    mapFile.readMapFile(mapPath)

    jsonStr = json.dumps(mapFile.toJson(humanReadable=humanReadable), indent=4)

    if outputPath is None:
        print(jsonStr)
    else:
        outputPath.parent.mkdir(parents=True, exist_ok=True)
        outputPath.write_text(jsonStr)

    return 0


def processArguments(args: argparse.Namespace, decompConfig: decomp_settings.Config|None=None):
    if decompConfig is not None:
        version = decompConfig.get_version_by_name(args.version)
        assert version is not None, f"Invalid version '{args.version}' selected"

        mapPath = Path(version.paths.map)
    else:
        mapPath = args.mapfile

    outputPath: Path|None = Path(args.output) if args.output is not None else None
    machine: bool = args.machine
    applyFixes: bool = args.apply_fixes

    exit(doJsonify(mapPath, outputPath, humanReadable=not machine, applyFixes=applyFixes))

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser], decompConfig: decomp_settings.Config|None=None):
    parser = subparser.add_parser("jsonify", help="Converts a mapfile into a json format.")

    emitMapfile = True
    if decompConfig is not None:
        versions = []
        for version in decompConfig.versions:
            versions.append(version.name)

        if len(versions) > 0:
            parser.add_argument("-v", "--version", help="Version to process from the decomp.yaml file", type=str, choices=versions, default=versions[0])
            emitMapfile = False

    if emitMapfile:
        parser.add_argument("mapfile", help="Path to a map file.", type=Path)

    parser.add_argument("-o", "--output", help="Output path of for the generated json. If omitted then stdout is used instead.")
    parser.add_argument("-m", "--machine", help="Emit numbers as numbers instead of outputting them as pretty strings.", action="store_true")
    parser.add_argument("-f", "--apply-fixes", help="DEPRECATED, this is applied automatically now. Apply certain fixups, like fixing size calculation of because of the existence of fake `.NON_MATCHING` symbols.", action="store_true")

    parser.set_defaults(func=processArguments)
