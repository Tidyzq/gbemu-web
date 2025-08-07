macro_rules! array {
    ($def:expr; $len:expr; $([$idx:expr]=$val:expr),* $(,)?) => { {
        let mut a = [$def; $len];
        $(a[$idx] = $val;)*
        a
    } }
}

pub(super) use array;

macro_rules! array_map {
    ($def:expr; $len:expr; $($val:expr),* $(,)?) => { {
        let mut a = [$def; $len];
        let mut idx = 0;
        $(
            a[idx] = $val;
            idx += 1;
        )*
        a
    } }
}

pub(super) use array_map;
