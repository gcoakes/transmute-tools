// Copyright 2021 Gregory Oakes
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#[macro_use]
extern crate transmute_tools;

#[cfg_attr(target_endian = "big", endianness(le))]
#[cfg_attr(target_endian = "little", endianness(be))]
#[derive(Default)]
#[repr(C, packed)]
pub struct Convert {
    pub a: u8,
    pub b: u8,
    pub c: u16,
    #[le]
    pub d: u128,
    pub e: Box<()>,
}

#[test]
fn conversion() {
    let mut convert = Convert::default();
    convert.set_c(10);
    let c = convert.c;
    assert_ne!(c, convert.c());
    convert.set_d(10);
    let d = convert.d;
    assert_eq!(d, convert.d());
}
