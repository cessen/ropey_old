# rope-rs

Rope-rs provides a unicode-aware implementation of mutable text ropes for Rust.  It does not currently provide the persistant data structure variant of ropes.  It is essentially intended as an alternative to Rust strings, where the contained text is expected to be large and frequently modified.

## Goals

- Unicode support on par with strings in Rust's standard library.  Should be able to index into and iterate over a rope by both chars and graphemes.
- Line aware.  Should be able to query information about line endings in various useful ways.
- Efficient manipulation of very large texts (at least up to hundreds of megabytes large), even with incoherent access patterns.

## Current Status

Rope-rs currently meets two of the above goals: it is line aware and can handle manipulation of very large texts efficiently.

Its unicode support is not yet where it needs to be.  It currently only indexes by graphemes, and there are corner-cases even for graphemes that are not handled correctly yet.
