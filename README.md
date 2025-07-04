# mapfile_parser

[![PyPI - Downloads](https://img.shields.io/pypi/dm/mapfile-parser)](https://pypi.org/project/mapfile-parser/)
[![GitHub License](https://img.shields.io/github/license/Decompollaborate/mapfile_parser)](https://github.com/Decompollaborate/mapfile_parser/releases/latest)
[![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/Decompollaborate/mapfile_parser)](https://github.com/Decompollaborate/mapfile_parser/releases/latest)
[![PyPI](https://img.shields.io/pypi/v/mapfile-parser)](https://pypi.org/project/mapfile-parser/)
![crate.io](https://img.shields.io/crates/dv/mapfile-parser)
[![GitHub contributors](https://img.shields.io/github/contributors/Decompollaborate/mapfile_parser?logo=purple)](https://github.com/Decompollaborate/mapfile_parser/graphs/contributors)

Map file parser library focusing decompilation projects.

This library is available for Python3 and Rust

## Features

- Fast parsing written in Rust.
- Support map formats:
  - GNU ld
  - clang lld
  - Metrowerks ld
- Built-in cli utilities to process the parsed map file (see [Examples](#examples)).

## Installing

### Python version

See this package at <https://pypi.org/project/mapfile_parser/>.

The recommended way to install is using from the PyPi release, via `pip`:

```bash
python3 -m pip install -U mapfile_parser
```

If you use a `requirements.txt` file in your repository, then you can add
this library with the following line:

```txt
mapfile_parser>=2.9.4,<3.0.0
```

#### Development version

The unstable development version is located at the [develop](https://github.com/Decompollaborate/mapfile_parser/tree/develop)
branch. PRs should be made into that branch instead of the main one.

The recommended way to install a locally cloned repo is by passing the `-e`
(editable) flag to `pip`.

```bash
python3 -m pip install -e .
```

In case you want to mess with the latest development version without wanting to
clone the repository, then you could use the following command:

```bash
python3 -m pip uninstall mapfile_parser
python3 -m pip install git+https://github.com/Decompollaborate/mapfile_parser.git@develop
```

NOTE: Installing the development version is not recommended unless you know what
you are doing. Proceed at your own risk.

### Rust version

See this crate at <https://crates.io/crates/mapfile_parser>.

To add this library to your project using Cargo:

```bash
cargo add mapfile_parser
```

Or add the following line manually to your `Cargo.toml` file:

```toml
mapfile_parser = "2.9.4"
```

## Versioning and changelog

This library follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
We try to always keep backwards compatibility, so no breaking changes should
happen until a major release (i.e. jumping from 2.X.X to 3.0.0).

To see what changed on each release check either the [CHANGELOG.md](CHANGELOG.md)
file or check the [releases page on Github](https://github.com/Decompollaborate/mapfile_parser/releases).
You can also use [this link](https://github.com/Decompollaborate/mapfile_parser/releases/latest)
to check the latest release.

## Examples

Various cli examples are provided in the [frontends folder](src/mapfile_parser/frontends).
Most of them are re-implementations of already existing tools using this
library to show how to use this library and inspire new ideas.

The list can be checked in runtime with `python3 -m mapfile_parser --help`.

Each one of them can be executed with `python3 -m mapfile_parser utilityname`,
for example `python3 -m mapfile_parser pj64_syms`.

- `bss_check`: Check that globally visible bss has not been reordered.
- `first_diff`: Find the first difference(s) between the built ROM and the base
  ROM.
- `jsonify`: Converts a mapfile into a json format.
- `pj64_syms`: Produce a PJ64-compatible symbol map.
- `objdiff_report`: Computes current progress of the matched functions. Expects
  `.NON_MATCHING` marker symbols on the mapfile to know which symbols are not
  matched yet.
- `progress`: Computes current progress of the matched functions. Relies on a
  [splat](https://github.com/ethteck/splat) folder structure and each matched
  functions no longer having an `.s` file (i.e: delete the file after matching it).
- `sym_info`: Display various information about a symbol or address.
- `symbol_sizes_csv`: Produces a csv summarizing the files sizes by parsing a
  map file.
- `upload_frogress`: Uploads current progress (calculated by the `progress`
  utility) of the matched functions to [frogress](https://github.com/decompals/frogress).

All these utilities support automatic scanning for a `decomp.yaml` file from
the [`decomp_settings`](https://github.com/ethteck/decomp_settings/) project.
This is the recommended way to use these utilities, because otherwise they need
a large number of long parameters to work. They are not meant to be used
directly, instead it is recommended to write a small script around them if a
project does not desire to adopt the `decomp.yaml` format.
