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

#![allow(dead_code)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

mod structurally_asserted;
use structurally_asserted::{StructurallyAsserted, StructurallyAssertedAttrs};

#[proc_macro_attribute]
pub fn test_structure(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(tokens as StructurallyAsserted);
    let attrs = parse_macro_input!(attrs as StructurallyAssertedAttrs);
    let size_assertion = attrs.into_assertions(&item.ident);
    let loc_assertions = item.drain_loc_assertions();
    let func_name = format_ident!("structure_{}", item.ident);
    let output = quote! {
        #item

        #[cfg(test)]
        #[allow(non_snake_case)]
        #[test]
        fn #func_name() {
            #loc_assertions
            #size_assertion
        }
    };
    output.into()
}
