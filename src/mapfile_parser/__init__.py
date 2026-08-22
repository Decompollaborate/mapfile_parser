#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

__version_info__ = (2, 13, 2)
__version__ = ".".join(map(str, __version_info__))  # + "-dev0"
__author__ = "Decompollaborate"

from . import frontends as frontends
from . import utils as utils
from .mapfile import FoundSymbolInfo as FoundSymbolInfo
from .mapfile import MapFile as MapFile
from .mapfile import MapsComparisonInfo as MapsComparisonInfo
from .mapfile import ReportCategories as ReportCategories
from .mapfile import Section as Section
from .mapfile import Segment as Segment
from .mapfile import Symbol as Symbol
from .mapfile import SymbolComparisonInfo as SymbolComparisonInfo
from .progress_stats import ProgressStats as ProgressStats

# Renamed types.
# TODO: remove on version 3.0
File = Section
