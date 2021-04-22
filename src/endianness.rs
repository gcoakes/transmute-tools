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

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use std::collections::HashSet;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token, Attribute, Field, Ident, Path, Token, Type, Visibility,
};

pub struct Endianness {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub brace_token: token::Brace,
    pub fields: Punctuated<Field, Token![,]>,
}

impl Endianness {
    pub fn drain_field_endianness(&mut self, default_endianness: Ident) -> TokenStream {
        let allowed_idents: HashSet<Ident> = [
            "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128",
        ]
        .iter()
        .map(|s| format_ident!("{}", s))
        .collect();
        let mut stream = TokenStream::new();
        for field in self.fields.iter_mut() {
            if field.ident.is_none() {
                stream.append_all(
                    syn::Error::new(
                        field.ident.span(),
                        "specifying endianness on tuple structs is unsupported",
                    )
                    .into_compile_error(),
                );
                continue;
            }
            let ident = field.ident.clone().unwrap();
            let vis = field.vis.clone();
            let typ = field.ty.clone();
            if let Type::Path(type_path) = &typ {
                let field_endianness = field
                    .attrs
                    .iter()
                    .filter(|a| a.path.is_ident("be") || a.path.is_ident("le"))
                    .map(|attr| attr.path.get_ident().unwrap())
                    .nth(0);
                if !type_path
                    .path
                    .get_ident()
                    .map(|i| allowed_idents.contains(i))
                    .unwrap_or(false)
                {
                    if field_endianness.is_some() {
                        stream.append_all(
                            syn::Error::new(
                                typ.span(),
                                "field specific endianness on a non-primitive",
                            )
                            .into_compile_error(),
                        );
                    }
                    continue;
                }
                field.vis = Visibility::Inherited;
                let endianness = field_endianness.unwrap_or(&default_endianness);
                let from_fn = format_ident!("from_{}", endianness);
                let to_fn = format_ident!("to_{}", endianness);
                let setter_ident = format_ident!("set_{}", ident);
                stream.append_all(quote! {
                    #[inline]
                    #vis fn #ident(&self) -> #typ {
                        #typ::#from_fn(self.#ident)
                    }
                    #[inline]
                    #vis fn #setter_ident(&mut self, value: #typ) {
                        self.#ident = value.#to_fn();
                    }
                });
            }
            field.attrs.retain(|attr| attr.path.is_ident("endianness"));
        }
        stream
    }
}

impl Parse for Endianness {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            let content;
            Ok(Self {
                attrs,
                vis,
                struct_token: input.parse()?,
                ident: input.parse()?,
                brace_token: braced!(content in input),
                fields: content.parse_terminated(Field::parse_named)?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Endianness {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            vis, ident, fields, ..
        } = self;
        tokens.append_all(&self.attrs);
        tokens.append_all(quote! {
            #vis struct #ident {
                #fields
            }
        });
    }
}

pub struct EndiannessAttrs(pub Ident);

impl Parse for EndiannessAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: Path = input.parse()?;
        if path.is_ident("le") || path.is_ident("be") {
            Ok(Self(path.get_ident().unwrap().clone()))
        } else {
            Err(syn::Error::new(
                path.span(),
                "endianness must either be le or be",
            ))
        }
    }
}
