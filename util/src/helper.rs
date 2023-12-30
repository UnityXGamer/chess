macro_rules! impl_conv {
    ($($impl_t:ty, $conv_t:ty, $from_fn_name:ident, $to_fn_name:ident, $($val:literal=$member:tt),+)&*) => {
        $(impl $impl_t {
            pub const fn $from_fn_name(input: $conv_t) -> Option<Self> {
                match input {
                    $($val => Some(Self::$member),)*
                    _ => None
                }
            }
            pub const fn $to_fn_name(&self) -> $conv_t {
                match self {
                    $(Self::$member => $val,)*
                }
            }
        })*
    };
}

pub(crate) use impl_conv;
