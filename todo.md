# TODO General Topics

- ArgsParser document (with Tool)
- ParamsGen EXE_NAME depending on app
- branch bench
- branch tests
  - $ cargo run -- cmp old.txt new.txt -n1 other result
  - $ cargo run -- cmp old.txt new.txt -bln50
- integration.rs: adjust to new error messages
  \*equirements Param
- Return String for Help but mark as OK. Probably Enum String or Params.
- Separation of concerns, no output or exit of the app
- Reusable components

# Open PRs

- PR 183 - branch u64/u128
- PR 185 - Divan Benchmark
- PR 159 - sdiff other with PR 188
- PR 187 - sdiff own with merged code of PR 159/188

# GNU Incompatibility
* single param with value -n50, -bln50, --bytes=50 --bytes 50
* unrecognized option double quotes instead of single quotes
