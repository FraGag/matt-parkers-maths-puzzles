# Solutions to Matt Parker's Maths Puzzles in Rust

These are my solutions to [Matt Parker's Maths Puzzles][maths-puzzles]
written in the [Rust programming language][rust-lang].

The solutions are wrapped up in a single executable.
The puzzle can be selected with a command-line argument.
Run the program with `--help` to see the puzzle names.
Run the program with `--help` after the puzzle name
to see additional parameters for solving a variant of the puzzle.

To build the program, you need Cargo,
which should come with the Rust compiler.
You can also run the program easily with Cargo:

```
cargo run -- --help
cargo run -- <puzzle-name>
cargo run -- <puzzle-name> --help
```

Unit tests can be executed with:

```
cargo test
```

[maths-puzzles]: http://www.think-maths.co.uk/maths-puzzles
[rust-lang]: https://www.rust-lang.org/

## License

The source code in this repository is released to the public domain:

<p xmlns:dct="http://purl.org/dc/terms/" xmlns:vcard="http://www.w3.org/2001/vcard-rdf/3.0#">
    <a rel="license" href="https://creativecommons.org/publicdomain/zero/1.0/">
        <img src="https://licensebuttons.net/p/zero/1.0/88x31.png" style="border-style: none;" alt="CC0" />
    </a>
    <br />
    To the extent possible under law,
    <a rel="dct:publisher" href="https://github.com/FraGag/matt-parkers-maths-puzzles">
        <span property="dct:title">Francis Gagné</span></a>
    has waived all copyright and related or neighboring rights to
    <span property="dct:title">Solutions to Matt Parker's Maths Puzzles in Rust</span>.
    This work is published from:
    <span property="vcard:Country" datatype="dct:ISO3166" content="CA"
        about="https://github.com/FraGag/matt-parkers-maths-puzzles">
        Canada</span>.
</p>

However, the program depends on third-party libraries that are not in the public domain.
Distribution of compiled executables must comply with the licenses of these third-party libraries.
