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

impl<T: 'static + NoGc> Type for T {
    type T = T;
}
impl<T: 'static + NoGc> Life for T {
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
    *a == *b; //~ [rustc E0623] [E] lifetime mismatch ...but data from `b` flows into `a` here
    a == b
}

fn good_concreate<'a, 'b, T: Life>(a: Gc<'a, usize>, b: Gc<'b, usize>) -> bool {
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

fn foo<'a, 'b, T: Life + Eq>(a: T::L<'a>, b: Gc<'b, ListF<T>>) {
    let a: List<'_, T> = List::Cons(a, b); //~ [rustc E0624] [E] lifetime mismatch ...but data from `b` flows into `a` here
}

fn foo_usize<'a, 'b>(a: <usize as Life>::L<'a>, b: Gc<'b, ListF<usize>>) -> bool {
    let a: List<'_, usize> = List::Cons(a, b);
    a == *b
}

fn foo_prim<'a, 'b, T: 'static + NoGc + Eq>(a: <T as Life>::L<'a>, b: Gc<'b, ListF<T>>) -> bool {
    let a: List<'_, T> = List::Cons(a, b);
    a == *b
}

mod map {
    use crate::*;
    struct Map<'r, K: Life, V: Life>(Option<Gc<'r, NodeF<K, V>>>);
    struct Node<'r, K: Life, V: Life> {
        key: K::L<'r>,
        size: usize,
        left: Map<'r, K, V>,
        right: Map<'r, K, V>,
        value: V::L<'r>,
    }
    pub struct MapF<K, V>(PhantomData<GcF<(K, V)>>);
    pub struct NodeF<K, V>(PhantomData<GcF<(K, V)>>);

    impl<'r, K: Life, V: Life> Type for Map<'r, K, V> {
        type T = MapF<K, V>;
    }
    impl<K: Life, V: Life> Life for MapF<K, V> {
        type L<'l> = Map<'l, K, V>;
    }

    impl<'r, K: Life, V: Life> Type for Node<'r, K, V> {
        type T = NodeF<K, V>;
    }
    impl<K: Life, V: Life> Life for NodeF<K, V> {
        type L<'l> = Node<'l, K, V>;
    }
}
