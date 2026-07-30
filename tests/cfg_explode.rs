#![allow(unexpected_cfgs)]

use pin_init::*;

macro_rules! explode {
    ($($field:ident)*) => {
        #[pin_data]
        pub struct Struct {
            $(
                #[cfg($field)]
                $field: u32,
            )*
        }

        fn init_struct() -> impl PinInit<Struct> {
            pin_init!(Struct {
                $(
                    #[cfg($field)]
                    $field <- 1,
                )*
            })
        }
    };
}

explode!(a b c d e f g h i j k l m n o p q r s t u v w x y z);

#[test]
fn cfg_explode() {
    stack_pin_init!(let _s = init_struct());
}
