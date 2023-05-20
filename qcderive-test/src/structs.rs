//! Testing all possible ways to write a `struct`.

#![allow(dead_code)]

use qcderive::QuickCheck;

#[derive(Clone, Debug, QuickCheck)]
struct StructNoneSemicolon;

#[derive(Clone, Debug, QuickCheck)]
#[allow(clippy::empty_structs_with_brackets)]
struct BraceStructEmpty {}

#[derive(Clone, Debug, QuickCheck)]
struct BraceStructSingleton<A> {
    a: A,
}

#[derive(Clone, Debug, QuickCheck)]
struct BraceStructMany<A, B, C> {
    a: A,
    b: B,
    c: C,
}

#[derive(Clone, Debug, QuickCheck)]
#[allow(clippy::empty_structs_with_brackets)]
struct TupleStructEmpty();

#[derive(Clone, Debug, QuickCheck)]
struct TupleStructSingleton<A>(A);

#[derive(Clone, Debug, QuickCheck)]
struct TupleStructMany<A, B, C>(A, B, C);

#[derive(Clone, Debug, QuickCheck)]
struct WithConstParam<T, const N: usize>(T, T, T);

// QuickCheck disallows non-static lifetimes
