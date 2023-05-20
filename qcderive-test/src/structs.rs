//! Testing all possible ways to write a `struct`.

use qcderive::QuickCheck;

#[derive(Clone, QuickCheck)]
struct StructNoneSemicolon;

#[derive(Clone, QuickCheck)]
#[allow(clippy::empty_structs_with_brackets)]
struct BraceStructEmpty {}

#[derive(Clone, QuickCheck)]
struct BraceStructSingleton<A> {
    a: A,
}

#[derive(Clone, QuickCheck)]
struct BraceStructMany<A, B, C> {
    a: A,
    b: B,
    c: C,
}

#[derive(Clone, QuickCheck)]
struct TupleStructEmpty();

#[derive(Clone, QuickCheck)]
struct TupleStructSingleton<A>(A);

#[derive(Clone, QuickCheck)]
struct TupleStructMany<A, B, C>(A, B, C);

#[derive(Clone, QuickCheck)]
struct WithConstParam<T, const N: usize>([T; N]);

// QuickCheck disallows non-static lifetimes
