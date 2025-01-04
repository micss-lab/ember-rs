#![no_std]

#[cfg(not(target_os = "none"))]
extern crate std;

macro_rules! tests {
    ($($test:ident),*) => {
        $(
            #[cfg(all(test, not(target_os = "none")))]
            mod $test;
        )*
    };
}

tests![parallel];
