#![allow(unexpected_cfgs)]

use pin_init::*;

// `#[pin_data]` and `[pin_]init!` resolve field cfgs by re-invoking themselves once per `cfg`'d
// field. Only one of the two generated branches is ever expanded, so this stays linear; were it
// exponential in the number of `cfg`s, this test would not finish.
macro_rules! explode {
    ($($field:ident)*) => {
        #[pin_data]
        pub struct Tuple(
            $(
                #[cfg($field)]
                u32,
            )*
            u32,
        );

        fn init_tuple() -> impl PinInit<Tuple> {
            pin_init!(Tuple(
                $(
                    #[cfg($field)]
                    1,
                )*
                0,
            ))
        }
    };
}

explode!(a b c d e f g h i j k l m n o p q r s t u v w x y z);

#[test]
fn cfg_explode() {
    stack_pin_init!(let tuple = init_tuple());
    assert_eq!(tuple.as_ref().get_ref().0, 0);
}
