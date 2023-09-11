#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2023 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import json
from pathlib import Path

from .. import mapfile


def doJsonify(mapPath: Path, outputPath: Path|None, humanReadable: bool=True) -> int:
    mapFile = mapfile.MapFile()
    mapFile.readMapFile(mapPath)

    jsonStr = json.dumps(mapFile.toJson(humanReadable=humanReadable), indent=4)

    if outputPath is None:
        print(jsonStr)
    else:
        outputPath.parent.mkdir(parents=True, exist_ok=True)
        outputPath.write_text(jsonStr)

    return 0


def processArguments(args: argparse.Namespace):
    mapPath: Path = args.mapfile
    outputPath: Path|None = Path(args.output) if args.output is not None else None
    machine: bool = args.machine

    exit(doJsonify(mapPath, outputPath, humanReadable=not machine))

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser]):
    parser = subparser.add_parser("jsonify", help="Converts a mapfile into a json format.")

    parser.add_argument("mapfile", help="Path to a map file", type=Path)
    parser.add_argument("-o", "--output", help="Output path of for the generated json. If omitted then stdout is used instead.")
    parser.add_argument("-m", "--machine", help="Emit numbers as numbers instead of outputting them as pretty strings.", action="store_true")

    parser.set_defaults(func=processArguments)
