#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import decomp_settings

import mapfile_parser


def mapfileParserMain():
    decompConfig: decomp_settings.Config|None
    try:
        decompConfig = decomp_settings.scan_for_config()
    except:
        decompConfig = None

    description = description="""\
Interface to call any of the mapfile_parser's CLI utilities.

All the CLI utilities support the `decomp.yaml` specification from the
[`decomp_settings`](https://github.com/ethteck/decomp_settings) project.

If a `decomp.yaml` file is detected, then every CLI argument that can be
inferred from that file will be be considered optional instead.

Most CLI utilites will also add a new optional "version" argument to allow
picking the version to process from the `decomp.yaml` file. It defaults to the
first listed version.
"""
    parser = argparse.ArgumentParser(
        description=description,
        prog="mapfile_parser",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument("-V", "--version", action="version", version=f"%(prog)s {mapfile_parser.__version__}")

    subparsers = parser.add_subparsers(description="action", help="the action to perform", required=True)

    mapfile_parser.frontends.bss_check.addSubparser(subparsers, decompConfig)
    mapfile_parser.frontends.first_diff.addSubparser(subparsers, decompConfig)
    mapfile_parser.frontends.jsonify.addSubparser(subparsers, decompConfig)
    mapfile_parser.frontends.objdiff_report.addSubparser(subparsers, decompConfig)
    mapfile_parser.frontends.pj64_syms.addSubparser(subparsers, decompConfig)
    mapfile_parser.frontends.progress.addSubparser(subparsers, decompConfig)
    mapfile_parser.frontends.sym_info.addSubparser(subparsers, decompConfig)
    mapfile_parser.frontends.symbol_sizes_csv.addSubparser(subparsers, decompConfig)
    mapfile_parser.frontends.upload_frogress.addSubparser(subparsers, decompConfig)

    args = parser.parse_args()
    args.func(args, decompConfig)


if __name__ == "__main__":
    mapfileParserMain()
