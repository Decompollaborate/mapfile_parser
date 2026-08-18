#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)


def convertibleToInt(value, base: int = 10) -> bool:
    try:
        int(value, base)
        return True
    except ValueError:
        return False


def readFileAsBytearray(filepath: Path) -> bytearray:
    if not filepath.exists():
        return bytearray(0)
    with filepath.open(mode="rb") as f:
        return bytearray(f.read())


def hexbytes(bs: bytes, addColons: bool = True) -> str:
    glue = ""
    if addColons:
        glue = ":"
    return glue.join(f"{c:02X}" for c in bs)


def getGitCommitTimestamp() -> int:
    return int(
        subprocess.check_output(["git", "show", "-s", "--format=%ct"])
        .decode("ascii")
        .rstrip()
    )


def getGitCommitHash() -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"]).decode("ascii").strip()


def generateFrogressEndpointUrl(baseurl: str, project: str, version: str) -> str:
    return "/".join([baseurl, "data", project, version]) + "/"


def getFrogressCategoriesDict(categories: dict) -> dict:
    return {
        "timestamp": getGitCommitTimestamp(),
        "git_hash": getGitCommitHash(),
        "categories": categories,
    }


def getFrogressDataDict(apikey: str, entries: dict) -> dict:
    return {
        "api_key": apikey,
        "entries": [entries],
    }
