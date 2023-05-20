# qcderive: `#[derive(QuickCheck)]`

Automatically implements `quickcheck::Arbitrary` for any data structure.

Rules:
- For `struct`s (or, generally, when you have _all_ of a collection of types), we simply call `quickcheck::Arbitrary::arbitrary` on each.
- For `enum`s (or, generally, when you have _one_ of a collection of types), we weight all variants equally.
- All type parameters (`<A, ...>`) must implement `quickcheck::Arbitrary`. If not, the struct will still work outside `quickcheck`, but you can't property-test it.
    - Caveat: We might in the future check if you actually use that type parameter, but for now, we don't (e.g. `PhantomData<A>` still requires `<A: Arbitrary>`).

## Syntax

```rust
//              vvvvvvvvvv
#[derive(Clone, QuickCheck)]
struct StructWithABunchOfEdgeCases<A, B, T, const N: usize> {
    a: A,
    b: B,
    t1: T,
    t2: T,
    t3: T,
}
```

automatically writes the following:

```rust
impl<
    A: ::quickcheck::Arbitrary,
    B: ::quickcheck::Arbitrary,
    T: ::quickcheck::Arbitrary,
    const N: usize, // recognizes this is not a type
> ::quickcheck::Arbitrary for StructWithABunchOfEdgeCases<A, B, T, { N }> {
    #[inline]
    fn arbitrary(g: &mut ::quickcheck::Gen) -> Self {
        a: <A as ::quickcheck::Arbitrary>::arbitrary(g),
        b: <B as ::quickcheck::Arbitrary>::arbitrary(g),
        t1: <T as ::quickcheck::Arbitrary>::arbitrary(g),
        t2: <T as ::quickcheck::Arbitrary>::arbitrary(g),
        t3: <T as ::quickcheck::Arbitrary>::arbitrary(g),
    }
}
```

`enum`s are in the works, then `union`s.

All credit for the incredible `quickcheck` library goes to its authors, not me! :)
