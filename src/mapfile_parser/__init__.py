#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

__version_info__ = (0, 0, 1)
__version__ = ".".join(map(str, __version_info__)) + "-dev"
__author__ = "Decompollaborate"

from .mapfile import MapFile
from .mapfile import Symbol, File

from . import frontends
