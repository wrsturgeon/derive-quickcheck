/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Testing all possible ways to write a `struct`.

#![allow(clippy::tests_outside_test_module, dead_code)]

use derive_quickcheck::QuickCheck;

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

#[derive(Clone, Debug, QuickCheck)]
struct LinkedList(Option<Box<LinkedList>>);

// Doesn't fail but takes a ridiculously long time
/*
#[derive(Clone, Debug, QuickCheck)]
struct Explosion(Vec<Explosion>);
*/

// QuickCheck disallows non-static lifetimes
