# structural-assert

Proc macros and traits to assist with safely creating transmutable data
structures. To be clear, this does not make it safe to use transmute. This is
just a collection of tools that might help you do that. The specific, initial
use case was to help faithfully reproduce structures from the [NVMe
spec](https://nvmexpress.org/specifications/).

## Usage

### Location Assertions

`transmute_tools::test_structure` provides an attribute which allows
specifying the location within a struct that a field must be placed:

``` rust
use structural_assert::test_structure;

#[test_structure(size = 20)]
#[repr(C, packed)]
pub struct Foo {
    #[loc(0:0)]
    pub a: u8,
    #[loc(1:1)]
    pub b: u8,
    #[loc(2:3)]
    pub c: u16,
    #[loc(4:19)]
    pub d: u128,
}
```

### Endianness

`transmute_tools::endianness` provides an attribute which allows specifying
specific endianness for a struct's fields. A struct like this:

``` rust
#[macro_use]
extern crate transmute_tools;

#[endianness(be)]
#[repr(C, packed)]
pub struct Foo {
    pub a: u8,
    pub b: u8,
    pub c: u16,
    #[le]
    pub d: u128,
    pub e: Box<()>,
}
```

Will generate this:

``` rust
#[repr(C, packed)]
pub struct Convert {
    // Note the missing visbilities.
    a: u8,
    b: u8,
    c: u16,
    d: u128,
    pub e: Box<()>,
}
impl Convert {
    #[inline]
    pub fn a(&self) -> u8 {
        u8::from_be(self.a)
    }
    #[inline]
    pub fn set_a(&mut self, value: u8) {
        self.a = value.to_be();
    }
    #[inline]
    pub fn b(&self) -> u8 {
        u8::from_be(self.b)
    }
    #[inline]
    pub fn set_b(&mut self, value: u8) {
        self.b = value.to_be();
    }
    #[inline]
    pub fn c(&self) -> u16 {
        u16::from_be(self.c)
    }
    #[inline]
    pub fn set_c(&mut self, value: u16) {
        self.c = value.to_be();
    }
    #[inline]
    pub fn d(&self) -> u128 {
        u128::from_le(self.d)
    }
    #[inline]
    pub fn set_d(&mut self, value: u128) {
        self.d = value.to_le();
    }
}
```

**NOTE**: This must generate functions rather than use something like
[simple_endianness](https://crates.io/crates/simple_endian) because access to
fields of packed structs when done by value rather than reference.
