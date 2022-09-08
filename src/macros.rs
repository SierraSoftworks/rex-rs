#[macro_export]
macro_rules! hashset {
    ([$x:expr]) => {
        {
            let mut temp_set = std::collections::HashSet::new();
            for i in $x {
                temp_set.insert(i.into());
            }
            temp_set
        }
    };

    ( $( $x:expr ),* ) => {
        {
            let mut temp_set = std::collections::HashSet::new();
            $(
                temp_set.insert($x.into());
            )*
            temp_set
        }
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
    ($prefix:expr) => {
        format!("{}{}", $prefix, env!("CARGO_PKG_VERSION"))
    };
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! version {
    () => {
        "0.0.0-dev"
    };
    ($prefix:expr) => {
        format!("{}0.0.0-dev", $prefix)
    };
}
