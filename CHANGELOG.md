# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add `CHANGELOG.md`

## [2.1.4] - 2023-09-11

### Fixed

- Fix vrom calculation if the first symbol of a file is not available in the
  map file

## [1.3.2] - 2023-09-11

### Added

- Add version info to the cli
- Add small testing suite
- Add machine-friendly/non-human-readable option for json generation.
  - Numbers are outputted as real numbers instead of prettified strings

### Changed

- Allow csv conversion to be written to a file instead of only printing to stdout
- Output `none` instead of `"None"` for symbols with no vrom when generating
  json output.

### Fixed

- Fix vrom calculation if the first symbol of a file is not available in the
  map file

Full changes: <https://github.com/Decompollaborate/mapfile_parser/compare/702a73452059ce4e97cda011e09dc4ef2a7b9dec...ba444b0bbfdfad7fb07347bf656b7fd4381596fb>

## [2.1.3] - 2023-08-30

### Fixed

- Fix version number
  - pypi thought previous version was a prerelease instead of a full release

## [2.1.2] - 2023-08-30

### Added

- Add machine-friendly/non-human-readable option for json generation.
  - Numbers are outputted as real numbers instead of prettified strings
- Add some CI tests

### Changed

- Output `none` instead of `"None"` for symbols with no vrom when generating
  json output.
- Make dummy files for `*fill*` lines instead of adding them to the previous file.
- Don't drop the dummy segment if it actually has data on it

## [2.1.1] - 2023-08-15

### Fixed

- Fix off-by-one issue which was throwing away tons of data

## [2.1.0] - 2023-08-14

### Added

- Add `bss_check` frontend
  - Allows to search for reordered bss variables by comparing two map files.

### Changed

- Don't skip important lines in some kinds of map files.
  - This may produce map parsing to be a bit slower but it should work properly
    with more kinds of mapfiles

### Fixed

- Try to prevent crashes if a file is found before the first segment is found

## [2.0.1] - 2023-08-07

### Changed

- Makes `Symbol`, `File` and `Segment` hashable

## [1.3.1] - 2023-08-06

### Changed

- Make `Symbol` and `File` types hashable

## [2.0.0] - 2023-08-01

### Added

- `toJson` method which allow serializing map files into the json format.
- `jsonify` frontend which allows converting a mapfile into a json format from
  the CLI.

### Changed

- Change logic of `MapFile` so it can parse and organize each file in proper
  segments.
  - This breaks old ways of iterating the `MapFile` class. Now iterating it
    yields a `Segment`, iterating that yields a `File`.
- Rename `segmentType` to `sectionType` and `filterBySegmentType` to
  `filterBySectionType`.

## [1.3.0] - 2023-08-01

### Added

- Add function `toJson` to export map file as a json. It returns a `dict`.
- New `jsonify` frontend, which allows converting a map file in the cli.

## [1.2.1] - 2023-07-28

### Fixed

- Fix missing `:` colon even when passing `addColons=True` to the `first_diff`
  frontend

## [1.2.0] - 2023-07-28

### Added

- `first_diff` frontend:
  - Allow an optional bytes converter callback. It can be useful to perform
    analysis or instruction decoding on the caller side.
  - Parameter to toggle colons (`:`) in bytes difference output

## [1.1.5] - 2023-07-28

### Fixed

- Fix map parsing ignoring some `*fill*` entries
- Improve symbol info output a bit

## [1.1.4] - 2023-04-03

### Fixed

- Add missing `request` requirement

## [1.1.3] - 2023-02-22

### Added

- Add flag to enable debug prints
- Add flag to specify the path index

### Fixed

- Properly handle files with multiple extensions

## [1.1.2] - 2022-12-14

### Changed

- Modularize `upload_frogress` frontend

## [1.1.1] - 2022-12-14

### Added

- Add examples to README

## [1.1.0] - 2022-12-14

### Added

- Compute vrom addresses of symbols and files
- Provide various front-ends clis (see README for more info):
  - `first_diff`
  - `pj64_syms`
  - `progress`
  - `sym_info`
  - `symbol_sizes_csv`
  - `upload_frogress`

[Full changelog](https://github.com/Decompollaborate/mapfile_parser/compare/1.0.0...1.1.0)

## [1.0.0] - 2022-12-13

### Added

- Initial release

[unreleased]: https://github.com/Decompollaborate/mapfile_parser/compare/master...develop
[2.1.4]: https://github.com/Decompollaborate/mapfile_parser/compare/1.3.2...2.1.4
[1.3.2]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.3...1.3.2
[2.1.3]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.2...2.1.3
[2.1.2]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.1...2.1.2
[2.1.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.0...2.1.1
[2.1.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.0.1...2.1.0
[2.0.1]: https://github.com/Decompollaborate/mapfile_parser/compare/1.3.1...2.0.1
[1.3.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.0.0...1.3.1
[2.0.0]: https://github.com/Decompollaborate/mapfile_parser/compare/1.3.0...2.0.0
[1.3.0]: https://github.com/Decompollaborate/mapfile_parser/compare/1.2.1...1.3.0
[1.2.1]: https://github.com/Decompollaborate/mapfile_parser/compare/1.2.0...1.2.1
[1.2.0]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.5...1.2.0
[1.1.5]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.4...1.1.5
[1.1.4]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.3...1.1.4
[1.1.3]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.2...1.1.3
[1.1.2]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.1...1.1.2
[1.1.1]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.0...1.1.1
[1.1.0]: https://github.com/Decompollaborate/mapfile_parser/compare/1.0.0...1.1.0
[1.0.0]: https://github.com/Decompollaborate/mapfile_parser/releases/tag/1.0.0