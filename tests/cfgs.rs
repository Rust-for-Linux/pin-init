use pin_init::{pin_data, pin_init, stack_pin_init, PinInit};

#[pin_data]
pub struct Struct {
    #[cfg(kernel)]
    field_d: Field,
    #[cfg(not(kernel))]
    field_e: Field,
}

impl Struct {
    pub fn new() -> impl PinInit<Self> {
        pin_init!(Self {
            #[cfg(kernel)]
            field_d: Field {},
            #[cfg(not(kernel))]
            field_e: Field {},
        })
    }
}

struct Field {}

#[pin_data]
pub struct Struct2 {
    // Test for cases where the type is not even defined when cfg is not satisfied.
    #[cfg(any())]
    non_exist: NonExistentType,
}

#[pin_data]
pub struct TupleStruct(#[cfg(any())] Field, u32, u32);

impl TupleStruct {
    pub fn new() -> impl PinInit<Self> {
        pin_init!(Self {
            #[cfg(any())]
            0: Field,
            // Disabled fields don't occupy an index!
            0: 10,
            1: 20,
        })
    }

    pub fn new_from_constructor() -> impl PinInit<Self> {
        pin_init!(Self(
            #[cfg(any())]
            Field,
            10,
            20,
        ))
    }
}

#[test]
fn tuple_fields_cfg_renumber() {
    stack_pin_init!(let value = TupleStruct::new());
    let proj = value.project();
    assert_eq!(*proj.0, 10);
    assert_eq!(*proj.1, 20);
}
