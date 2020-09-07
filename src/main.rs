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
struct PF<T>(PhantomData<T>);

impl<T: 'static + NoGc> Type for T {
    type T = PF<T>;
}
impl<T: 'static + NoGc> Life for PF<T> {
    type L<'l> = T;
}

struct Gc<'r, T: Life>(&'r T::L<'r>);
struct GcF<T>(PhantomData<T>);

impl<'b, 'r, T:Life> PartialEq<Gc<'r, T>> for Gc<'r, T> where for<'l, 'll> T::L<'l>: PartialEq<T::L<'ll>> {
    fn eq(&self, other: &Gc<'r, T>) -> bool {
        self.0 == self.0
    }
}

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

enum List<'r, T: Life> {
    Cons(T::L<'r>, Gc<'r, ListF<T>>),
    Nil,
}
struct ListF<T: Life>(PhantomData<GcF<T>>);


impl<'r, T:Life> Eq for List<'r, T> where for<'l> T::L<'l>: Eq { }
impl<'r, T:Life> PartialEq for List<'r, T> where for<'l> T::L<'l>: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (List::Cons(a, a_next), List::Cons(b, b_next)) => a == b && *a_next == *b_next,
            _ => false,
        }
    }
}

impl<'r, T: Life> Type for List<'r, T> {
    type T = ListF<T>;
}
impl<T: Life> Life for ListF<T> {
    type L<'l> = List<'l, T>;
}

// fn foo<'l, 'a: 'l, 'b: 'l, T: Life + Eq>(a: T::L<'a>, b: Gc<'b, ListF<T>>) {
//     // let a: List<'l, T> = List::Cons(a, b); //~ [rustc E0623] [E] lifetime mismatch ...but data from `b` flows into `a` here
// }

// fn foo_usize<'a, 'b>(a: <PF<usize> as Life>::L<'a>, b: Gc<'b, ListF<PF<usize>>>) -> bool {
//     let a: List<'_, PF<usize>> = List::Cons(a, b);
//     a == *b
// }

// #[derive(PartialEq)]
// struct Foo<'r>(&'r Gc<'r, PF<String>>);
// #[derive(PartialEq)]
// struct FooF;
// impl<'r> Type for Foo<'r> {
//     type T = FooF;
// }
// impl Life for FooF {
//     type L<'l> = Foo<'l>;
// }

// fn foo_foo<'a, 'b>(a: Foo<'a>, b: Gc<'b, ListF<FooF>>) -> bool {
//     let a = List::Cons(a, b);
//     a == *b
// }

// #[derive(PartialEq)]
// struct Bar<'r, T: NoGc + 'static>(&'r Gc<'r, PF<T>>);
// #[derive(PartialEq)]
// struct BarF<T>(PhantomData<T>);
// impl<'r, T: NoGc + 'static> Type for Bar<'r, T> {
//     type T = BarF<T>;
// }
// impl<T: 'static + NoGc> Life for BarF<T> {
//     type L<'l> = Bar<'l, T>;
// }

// fn foo_bar<'a, 'b, T: NoGc + 'static + Eq>(a: Bar<'a, T>, b: Gc<'b, ListF<BarF<T>>>) -> bool {
//     let a = List::Cons(a, b);
//     a == *b
// }

// impl<'r, T: Life> PartialEq for BarGc<'r, T> where for<'l> T::L<'l>: PartialEq {
//     fn eq(&self, other: &Self) -> bool {
//         self.0 == self.0
//     }
// }

// struct BarGc<'r, T: Life>(&'r Gc<'r, T>);
// #[derive(PartialEq)]
// struct BarGcF<T>(PhantomData<T>);
// impl<'r, T: Life> Type for BarGc<'r, T> {
//     type T = BarGcF<T>;
// }
// impl<T: Life> Life for BarGcF<T> {
//     type L<'l> = BarGc<'l, T>;
// }

// fn foo_bar_maybe_gc<'a, 'b, T: Life + Eq>(a: BarGc<'a, T>, b: Gc<'b, ListF<BarGcF<T>>>) -> bool where for<'l> T::L<'l>: Eq {
//     let a = List::Cons(a, b);
//     a == *b
// }
