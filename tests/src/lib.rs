#![no_std]

#[cfg(not(target_os = "none"))]
#[macro_use]
extern crate std;

macro_rules! tests {
    ($($test:ident),*) => {
        $(
            #[cfg(all(test, not(target_os = "none")))]
            mod $test;
        )*
    };
}

tests![parallel, sequential];
