#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse

import mapfile_parser


def mapfileParserMain():
    parser = argparse.ArgumentParser()

    subparsers = parser.add_subparsers(description="action", help="the action to perform", required=True)

    mapfile_parser.frontends.function_sizes_csv.addSubparser(subparsers)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    mapfileParserMain()
