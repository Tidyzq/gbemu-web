macro_rules! array {
    ($def:expr; $len:expr; $([$idx:expr]=$val:expr),* $(,)?) => { {
        let mut a = [$def; $len];
        $(a[$idx] = $val;)*
        a
    } }
}

pub(super) use array;

macro_rules! bit {
    ($exp:expr, $n:literal) => {
        ($exp & (1 << $n)) != 0
    }
}

pub(super) use bit;

macro_rules! set_bit {
    ($exp:expr, $n:literal, $b:expr) => {{
        if $b {
            $exp = ($exp | (1 << $n))
        } else {
            $exp = ($exp & !(1 << $n))
        }
    }}
}

pub(super) use set_bit;
