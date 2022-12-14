#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse

import mapfile_parser


def mapfileParserMain():
    parser = argparse.ArgumentParser()

    subparsers = parser.add_subparsers(description="action", help="the action to perform", required=True)

    mapfile_parser.frontends.first_diff.addSubparser(subparsers)
    mapfile_parser.frontends.pj64_syms.addSubparser(subparsers)
    mapfile_parser.frontends.progress.addSubparser(subparsers)
    mapfile_parser.frontends.sym_info.addSubparser(subparsers)
    mapfile_parser.frontends.symbol_sizes_csv.addSubparser(subparsers)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    mapfileParserMain()
