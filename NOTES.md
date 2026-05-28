# One Billion Row Challenge Notes

## TODOs

* "Do math" to see if copyback penalty is expected
* Find indexes of semicolons and newlines
* Parse numbers and add them all up
* Store weather station names efficiently
* Come up with branchless way of tracking copyback_len

## Simd equivalent of `Iterator::position()`

For example, to find the position of a newline character in a SIMD register, do
the following:

```rust
const NEWLINES: std::simd::u8x32 = std::simd::u8x32::splat(b'\n');
let chunk_simd = std::simd::u8x32::from(*chunk);
let idx = std::simd::cmp::SimdPartialEq::simd_eq(chunk_simd, NEWLINES).first_set();
match idx {
    0 => semicolon_mask.shift_elements_left::<0>(false),
    1 => semicolon_mask.shift_elements_left::<1>(false),
    ...
}
```

Some notes:

* `Mask<T, N>` is array-like so the "first" set bit is farther to the "left"
  than other set bits.
* When you call `Mask<T, N>::to_bitmask()`, you get a `u64` where the "first"
  set bit is farther to the right than any other set bit.

## Finding ranges of bytes which represent numeric values

The basic loop in our heads is to make a mask which indicates all the values of
semicolons, make a mask which indicates all the values of newlines, OR them
together, and repeatedly call `first_set()` and `shift_elements_left()` tracking
whether the next element is a semicolon or a newline (recall from the format
that numbers are always between a semicolon and a newline).

It seems to us that `shift_elements_left()` doesn't work with a non-const index,
so we're not sure how that's going to work. Perhaps the match statement in the
code block above.
