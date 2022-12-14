#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from pathlib import Path
import requests # type: ignore

from .. import utils
from .. import progress_stats
from . import progress


def doUploadFrogress(mapPath: Path, asmPath: Path, nonmatchingsPath: Path, project: str, version: str, category: str, baseurl: str, apikey: str|None=None, verbose: bool=False) -> int:
    totalStats, progressPerFolder = progress.getProgress(mapPath, asmPath, nonmatchingsPath)

    entries: dict[str, int] = {}
    if verbose:
        progress_stats.ProgressStats.printHeader()
        totalStats.print("all", totalStats)
        print()

    entries.update(totalStats.getAsFrogressEntry("all"))

    for folder, statsEntry in progressPerFolder.items():
        if verbose:
            statsEntry.print(folder, totalStats)
        entries.update(statsEntry.getAsFrogressEntry(folder))

    if verbose:
        print()

    url = utils.generateFrogressEndpointUrl(baseurl, project, version)

    if verbose:
        print(f"Publishing entries to {url}")
        for key, value in entries.items():
            print(f"\t{key}: {value}")

    if not apikey:
        if verbose:
            print("Missing apikey, exiting without uploading")
        return 1

    data = utils.getFrogressDataDict(apikey, utils.getFrogressCategoriesDict({category: entries}))

    r = requests.post(url, json=data)
    r.raise_for_status()
    if verbose:
        print("Done!")

    return 0


def processArguments(args: argparse.Namespace):
    mapPath: Path = args.mapfile
    asmPath: Path = args.asmpath
    nonmatchingsPath: Path = args.nonmatchingspath
    project: str = args.project
    version: str = args.version
    category: str = args.category
    baseurl: str = args.baseurl
    apikey: str|None = args.apikey
    verbose: bool = args.verbose

    exit(doUploadFrogress(mapPath, asmPath, nonmatchingsPath, project, version, category, baseurl, apikey, verbose))

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser]):
    parser = subparser.add_parser("upload_frogress", help="Uploads current progress of the matched functions to frogress (https://github.com/decompals/frogress).")

    parser.add_argument("mapfile", help="Path to a map file", type=Path)
    parser.add_argument("asmpath", help="Path to asm folder", type=Path)
    parser.add_argument("nonmatchingspath", help="Path to nonmatchings folder", type=Path)
    parser.add_argument("project", help="Project slug")
    parser.add_argument("version", help="Version slug")
    parser.add_argument("category", help="Category slug")
    parser.add_argument("--baseurl", help="API base URL", default="https://progress.deco.mp")
    parser.add_argument("--apikey", help="API key. Dry run is performed if this option is omitted")
    parser.add_argument("-v", "--verbose", action="store_true")

    parser.set_defaults(func=processArguments)
