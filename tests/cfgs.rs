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

#[cfg(not(feature = "std"))]
fn assert_pinned<T>(_: core::pin::Pin<&mut T>) {}

#[pin_data]
pub struct Struct2 {
    // Test for cases where the type is not even defined when cfg is not satisfied.
    #[cfg(any())]
    non_exist: NonExistentType,
}

#[pin_data]
pub struct TupleStruct(Field, #[cfg(any())] HiddenField);

impl TupleStruct {
    pub fn new() -> impl PinInit<Self> {
        pin_init!(Self(Field {}))
    }
}

#[allow(dead_code)]
struct HiddenField;

#[test]
fn tuple_struct_allows_cfgd_out_last_field() {
    stack_pin_init!(let value = TupleStruct::new());
    let projected = value.as_mut().project();
    let _ = projected.0;
}

#[pin_data]
pub struct ConstructorCfgTuple(Field, #[cfg(any())] HiddenField);

impl ConstructorCfgTuple {
    pub fn new() -> impl PinInit<Self> {
        pin_init!(Self(
            Field {},
            #[cfg(any())]
            HiddenField
        ))
    }
}

#[test]
fn tuple_constructor_allows_cfgd_out_last_argument() {
    stack_pin_init!(let value = ConstructorCfgTuple::new());
    let projected = value.as_mut().project();
    let _ = projected.0;
}

#[pin_data]
pub struct FeatureTupleStruct(
    Field,
    #[cfg(not(feature = "std"))]
    #[pin]
    core::marker::PhantomPinned,
);

impl FeatureTupleStruct {
    pub fn new() -> impl PinInit<Self> {
        pin_init!(Self(
            Field {},
            #[cfg(not(feature = "std"))]
            core::marker::PhantomPinned
        ))
    }
}

#[test]
fn tuple_struct_allows_feature_cfgd_out_last_field() {
    stack_pin_init!(let value = FeatureTupleStruct::new());
    let projected = value.as_mut().project();
    let _ = projected.0;
    #[cfg(not(feature = "std"))]
    {
        assert_pinned(projected.1);
    }
}
