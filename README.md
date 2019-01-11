# selector

Provides a UI to select lines

## Command line usage

```sh
$ program_that_generate_lines | selector | program_that_accept_selected_lines_as_input
> [ ]  M src/main.rs
  [*] ?? a/new/file
  [ ] ?? another/new/file

```

## Library usage

```rust
extern crate selector;

fn main() {
    let lines: Vec<String> = vec!["First line".to_string(), "Second line".to_string()];

    let selected_lines: Vec<String> = selector::select(lines);
}

```