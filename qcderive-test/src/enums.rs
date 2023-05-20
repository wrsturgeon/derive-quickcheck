//! Testing all possible ways to write a `enum`.

#![allow(dead_code)]

use qcderive::QuickCheck;

#[derive(Clone, Debug, QuickCheck)]
enum EnumEmpty {}

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

// QuickCheck disallows non-static lifetimes
