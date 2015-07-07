# Ropey

Ropey provides a unicode-aware implementation of mutable text ropes for Rust.  It does not currently provide the persistant data structure variant of ropes.  It is essentially intended as an alternative to Rust strings, where the contained text is expected to be large and frequently modified.

## Goals

- Unicode support on par with strings in Rust's standard library.  Should be able to index into and iterate over a rope by both chars and graphemes.
- Line aware.  Should be able to query information about line endings in various useful ways.
- Efficient manipulation of very large texts (at least up to hundreds of megabytes large), even with incoherent access patterns.

## Current Status

Ropey currently meets all three goals for the most part, but there is still
much work to be done:

- You can convert an entire rope to a String, but you can't grab just part of
  it as a String.
- All the iterators are currently forward-only.  It would be great to make
  them bi-directional.
- The code could be cleaner and better organized.
- The leaf nodes of the rope can potentially get badly fragmented by certain
  operations.  This probably isn't a huge problem in practice, but it would
  nevertheless be good to improve the code to reduce this.
- There is lots of optimization potential!  Much of the code currently does
  a lot of redundant work, simply because it was the easiest way to code it.
  But now that everything appears to be working correctly and has lots of
  unit tests, it would be good to go back and start improving the code for
  performance.