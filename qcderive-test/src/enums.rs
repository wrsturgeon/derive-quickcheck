/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Testing all possible ways to write a `enum`.

#![allow(clippy::tests_outside_test_module, dead_code)]

use derive_quickcheck::QuickCheck;

// Empty `enum` is uninstantiable so can't be implemented

#[derive(Clone, Debug, QuickCheck)]
enum EnumSingletonNoMembers {
    OnlyOption,
}

#[derive(Clone, Debug, QuickCheck)]
enum EnumSingletonOneMember<A> {
    OnlyOption(A),
}

#[derive(Clone, Debug, QuickCheck)]
enum EnumSingletonManyMembers<A, B, C> {
    OnlyOption(A, B, C),
}

#[derive(Clone, Debug, QuickCheck)]
enum EnumManyNoMembers {
    First,
    Second,
    Third,
}

#[derive(Clone, Debug, QuickCheck)]
enum EnumManyOneMember<A, B, C> {
    First(A),
    Second(B),
    Third(C),
}

#[derive(Clone, Debug, QuickCheck)]
enum EnumManyManyMembers<A, B, C> {
    First(A, B, C),
    Second(A, B, C),
    Third(A, B, C),
}

#[derive(Clone, Debug, QuickCheck)]
enum LinkedList {
    End,
    More(Box<LinkedList>),
}

// Doesn't fail but takes a ridiculously long time
/*
#[derive(Clone, Debug, QuickCheck)]
enum Explosion {
    End,
    More(Vec<Explosion>),
}
*/

// QuickCheck disallows non-static lifetimes
