#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(marker_trait_attr)]
#![feature(generic_associated_types)]
#![feature(negative_impls)]
#![feature(optin_builtin_traits)]

fn main() {
    println!("Hello, world!");
}

use std::marker::PhantomData;

pub unsafe auto trait NoGc {}
impl<'r, T> !NoGc for Gc<'r, T> {}
impl<T> !NoGc for GcF<T> {}

trait Type {
    type T: 'static + Life;
}
trait Life: 'static {
    type L<'l>: Type<T = Self>;
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct PF<T: 'static + NoGc>(PhantomData<T>);

impl<T: 'static + NoGc> Type for T {
    type T = PF<T>;
}
impl<T: 'static + NoGc> Life for PF<T> {
    type L<'l> = T;
}

#[derive(Eq, PartialEq)]
struct Gc<'r, T: Life>(&'r T::L<'r>);
struct GcF<T>(PhantomData<T>);

impl<'r, T: Life> Copy for Gc<'r, T> {}
impl<'r, T: Life> Clone for Gc<'r, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'r, T: Life> std::ops::Deref for Gc<'r, T> {
    type Target = T::L<'r>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'r, T: Life> Type for Gc<'r, T> {
    type T = GcF<T>;
}
impl<T: Life> Life for GcF<T> {
    type L<'l> = Gc<'l, T>;
}

fn bad<'a, 'b, T: Life + Eq>(a: Gc<'a, T>, b: Gc<'b, T>) -> bool
where
    for<'l> T::L<'l>: Eq,
{
    // *a == *b; //~ [rustc E0623] [E] lifetime mismatch ...but data from `b` flows into `a` here
    a == b
}

fn good_concreate<'a, 'b, T: Life>(a: Gc<'a, PF<usize>>, b: Gc<'b, PF<usize>>) -> bool {
    let _ = a == b;
    *a == *b
}

fn good<T: Eq>(a: &T, b: &T) -> bool {
    let _ = a == b;
    *a == *b
}

#[derive(Eq, PartialEq)]
enum List<'r, T: Life> {
    Cons(T::L<'r>, Gc<'r, ListF<T>>),
    Nil,
}
struct ListF<T: Life>(PhantomData<GcF<T>>);

impl<T: Life> Eq for ListF<T> {}
impl<T: Life> PartialEq for ListF<T> {
    fn eq(&self, other: &Self) -> bool {
        unreachable!()
    }
}

impl<'r, T: Life> Type for List<'r, T> {
    type T = ListF<T>;
}
impl<T: Life> Life for ListF<T> {
    type L<'l> = List<'l, T>;
}

fn foo<'l, 'a: 'l, 'b: 'l, T: Life + Eq>(a: T::L<'a>, b: Gc<'b, ListF<T>>) {
    // let a: List<'l, T> = List::Cons(a, b); //~ [rustc E0623] [E] lifetime mismatch ...but data from `b` flows into `a` here
}

fn foo_usize<'l, 'a: 'l, 'b: 'l>(
    a: <PF<usize> as Life>::L<'a>,
    b: Gc<'b, ListF<PF<usize>>>,
) -> bool {
    let a: List<'l, PF<usize>> = List::Cons(a, b);
    a == *b
}
