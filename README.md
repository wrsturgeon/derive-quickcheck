# qcderive: `#[derive(QuickCheck)]`

Automatically implements `quickcheck::Arbitrary` for any data structure.

Rules:
- For `struct`s (or, generally, when you have _all_ of a collection of types), we simply call `quickcheck::Arbitrary::arbitrary` on each.
- For `enum`s (or, generally, when you have _one_ of a collection of types), we weight all variants equally.

## Syntax

```rust
//              vvvvvvvvvv
#[derive(Clone, QuickCheck)]
struct StructWithEdgeCases<A, B, T, const N: usize>(A, B, DepType<T, { N }>);
```

automatically writes the following:

```rust
impl<
    A: ::quickcheck::Arbitrary,
    B: ::quickcheck::Arbitrary,
    T: ::quickcheck::Arbitrary,
    const N: usize,
> ::quickcheck::Arbitrary for StructWithEdgeCases<A, B, T, { N }> {
    #[inline]
    fn arbitrary(g: &mut ::quickcheck::Gen) -> Self {
        <A as ::quickcheck::Arbitrary>::arbitrary(g),
        <B as ::quickcheck::Arbitrary>::arbitrary(g),
        <DepType<T, { N }> as ::quickcheck::Arbitrary>::arbitrary(g),
    }
}
```

`enum`s are in the works, then `union`s.

All credit for the incredible `quickcheck` library goes to its authors, not me! :)
