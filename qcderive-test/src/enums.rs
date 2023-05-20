//! Testing all possible ways to write a `enum`.

#![allow(dead_code)]

use qcderive::QuickCheck;

#[derive(Clone, QuickCheck)]
enum EnumEmpty {}

#[derive(Clone, QuickCheck)]
enum EnumSingletonNoMembers {
    OnlyOption,
}

#[derive(Clone, QuickCheck)]
enum EnumSingletonOneMember<A> {
    OnlyOption(A),
}

#[derive(Clone, QuickCheck)]
enum EnumSingletonManyMembers<A, B, C> {
    OnlyOption(A, B, C),
}

#[derive(Clone, QuickCheck)]
enum EnumManyNoMembers {
    First,
    Second,
    Third,
}

#[derive(Clone, QuickCheck)]
enum EnumManyOneMember<A, B, C> {
    First(A),
    Second(B),
    Third(C),
}

#[derive(Clone, QuickCheck)]
enum EnumManyManyMembers<A, B, C> {
    First(A, B, C),
    Second(A, B, C),
    Third(A, B, C),
}

// QuickCheck disallows non-static lifetimes
