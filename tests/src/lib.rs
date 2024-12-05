#![cfg_attr(target_os = "none", no_std)]

macro_rules! tests {
    ($($test:ident),*) => {
        $(
            #[cfg(all(test, not(target_os = "none")))]
            mod $test;
        )*
    };
}

tests![parallel];
