# Orgize

Orgize is a Emacs Org-mode parser written by pure Rust. It behaves like a pull
parser (returning an iterator of events) but not exactly.

Besides, orgize also provides some mechanism for exporting org-mode files to
various formats, e.g. HTML.

## Usage

```toml
[dependencies]
orgize = "0.1.0"
```

```rust
// Rust 2015 only
extern crate orgize;
```

## Example

```rust
use orgize::Parser;

fn main() {
    let parser = Parser::new(
        r#"* Title 1
*Section 1*
** Title 2
_Section 2_
* Title 3
/Section 3/
* Title 4
=Section 4="#,
    );

    for event in parser {
        // handling the event
    }
}
```

Alternatively, you can use the built-in render.

## License

MIT