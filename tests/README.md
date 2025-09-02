# tests

## Adding a test

Copy the mapfile into a subdirectory of the `maps` folder. Top level folders of
`maps` corresponds to the linker used to generate the mapfile, inside goes the
platform with an optional version number, then the mapfile.

Make sure to install the develpment version of mapfile_parser (`pip install .`).

Run `./tests/update_outputs.py` from the root of the repository.

Commit all the generated files.
