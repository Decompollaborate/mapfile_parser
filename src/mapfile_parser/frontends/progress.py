#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from pathlib import Path

from .. import mapfile
from .. import progress_stats


def getProgress(mapPath: Path, asmPath: Path, nonmatchingsPath: Path) -> tuple[progress_stats.ProgressStats, dict[str, progress_stats.ProgressStats]]:
    mapFile = mapfile.MapFile()
    mapFile.readMapFile(mapPath)

    return mapFile.filterBySegmentType(".text").getProgress(asmPath, nonmatchingsPath)

def doProgress(mapPath: Path, asmPath: Path, nonmatchingsPath: Path) -> int:
    totalStats, progressPerFolder = getProgress(mapPath, asmPath, nonmatchingsPath)

    progress_stats.printStats(totalStats, progressPerFolder)
    return 0


def processArguments(args: argparse.Namespace):
    mapPath: Path = args.mapfile
    asmPath: Path = args.asmpath
    nonmatchingsPath: Path = args.nonmatchingspath

    exit(doProgress(mapPath, asmPath, nonmatchingsPath))

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser]):
    parser = subparser.add_parser("progress", help="Computes current progress of the matched functions. Relies on a splat (https://github.com/ethteck/splat) folder structure and matched functions not longer having a file.")

    parser.add_argument("mapfile", help="Path to a map file", type=Path)
    parser.add_argument("asmpath", help="Path to asm folder", type=Path)
    parser.add_argument("nonmatchingspath", help="Path to nonmatchings folder", type=Path)

    parser.set_defaults(func=processArguments)
