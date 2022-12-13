#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations


def convertibleToInt(value, base: int=10) -> bool:
    try:
        int(value, base)
        return True
    except ValueError:
        return False
