//! This crate provides function, `take()`.
//!
//! `take()` allows for taking `T` out of a `&mut T`, doing anything with it including consuming it, and producing another `T` to put back in the `&mut T`.
//!
//! During `take()`, if a panic occurs, the entire process will be exited, as there's no valid `T` to put back into the `&mut T`.
//!
//! Contrast with `std::mem::replace()`, which allows for putting a different `T` into a `&mut T`, but requiring the new `T` to be available before being able to consume the old `T`.
//!
//! The crate also provides `take_no_exit()` function, which behaves similarly but instead of exiting
//! the program on panic, it leaves a sentinel value there.

extern crate unreachable;

mod exit_on_panic;

use exit_on_panic::exit_on_panic;

/// Allows use of a value pointed to by `&mut T` as though it was owned, as long as a `T` is made available afterwards.
///
/// The closure must return a valid T.
/// # Important
/// Will exit the program (with status code 101) if the closure panics.
///
/// # Example
/// ```
/// struct Foo;
/// let mut foo = Foo;
/// take_mut::take(&mut foo, |foo| {
///     // Can now consume the Foo, and provide a new value later
///     drop(foo);
///     // Do more stuff
///     Foo // Return new Foo from closure, which goes back into the &mut Foo
/// });
/// ```
pub fn take<T, F>(mut_ref: &mut T, closure: F)
  where F: FnOnce(T) -> T {
    use std::ptr;
    exit_on_panic(|| {
        unsafe {
            let old_t = ptr::read(mut_ref);
            let new_t = closure(old_t);
            ptr::write(mut_ref, new_t);
        }
    });
}

/// Represents an invalid value that is safe to drop
pub trait Sentinel: Sized {
    /// Creates the sentinel.
    fn new_sentinel() -> Self;

    /// Releases the sentinel. Calling this indicates that nothing unexpected happened.
    /// The caller must make sure that the value this function is called with is the exact same
    /// value the `new_sentinel()` funtion returned.
    unsafe fn release_sentinel(self) {
    }
}

impl<T> Sentinel for Option<T> {
    fn new_sentinel() -> Self {
        None
    }

    unsafe fn release_sentinel(self) {
        // This avoids unnecessary check for None
        use unreachable::UncheckedOptionExt;
        self.unchecked_unwrap_none();
    }
}

/// This function is similar to `take()` but instead of exiting, it will leave sentinel value in
/// place of the original in case of panic.
pub fn take_no_exit<T, F>(mut_ref: &mut T, closure: F)
  where T: Sentinel,
        F: FnOnce(T) -> T {
    use std::mem::replace;
    unsafe {
        let old_t = replace(mut_ref, Sentinel::new_sentinel());
        let new_t = closure(old_t);
        replace(mut_ref, new_t).release_sentinel();
    }
}


#[test]
fn it_works() {
    #[derive(PartialEq, Eq, Debug)]
    enum Foo {A, B};
    impl Drop for Foo {
        fn drop(&mut self) {
            match *self {
                Foo::A => println!("Foo::A dropped"),
                Foo::B => println!("Foo::B dropped")
            }
        }
    }
    let mut foo = Foo::A;
    take(&mut foo, |mut f| {
       drop(f);
       Foo::B
    });
    assert_eq!(&foo, &Foo::B);
}
