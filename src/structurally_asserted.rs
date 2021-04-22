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
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token, Attribute, Field, Ident, Lit, LitInt, MetaNameValue, Token, Type, Visibility,
};

pub struct StructurallyAsserted {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub brace_token: token::Brace,
    pub fields: Punctuated<Field, Token![,]>,
}

impl StructurallyAsserted {
    pub fn drain_loc_assertions(&mut self) -> TokenStream {
        let mut stream = TokenStream::new();
        for field in self.fields.iter_mut() {
            for attr in field.attrs.iter() {
                if !attr.path.is_ident("loc") {
                    continue;
                }
                if field.ident.is_none() {
                    stream.append_all(
                        syn::Error::new(
                            field.ident.span(),
                            "structural assertions are not supported on tuple structs",
                        )
                        .to_compile_error(),
                    );
                }
                match attr.parse_args::<LocAttr>() {
                    Ok(loc) => stream.append_all(loc.into_assertions(
                        &self.ident,
                        field.ident.as_ref().unwrap(),
                        &field.ty,
                    )),
                    Err(e) => stream.append_all(e.to_compile_error()),
                }
            }
            field.attrs.retain(|attr| !attr.path.is_ident("loc"));
        }
        stream
    }
}

impl Parse for StructurallyAsserted {
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

impl ToTokens for StructurallyAsserted {
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

#[derive(Default)]
pub struct StructurallyAssertedAttrs {
    size: Option<LitInt>,
}

impl Parse for StructurallyAssertedAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let nvs = input.parse_terminated::<_, Token![,]>(MetaNameValue::parse)?;
        let mut res = Self::default();
        for nv in nvs {
            if nv.path.is_ident("size") {
                res.size = match nv.lit {
                    Lit::Int(lit) => Some(lit),
                    lit => {
                        return Err(syn::Error::new(
                            lit.span(),
                            "size must be an integer literal",
                        ))
                    }
                };
            }
        }
        Ok(res)
    }
}

impl StructurallyAssertedAttrs {
    pub fn into_assertions(self, ident: &Ident) -> TokenStream {
        let size_assertion = self.size.map(|size| quote! {
            assert_eq!(#size, ::core::mem::size_of::<#ident>(), "size of {}", ::core::stringify!(#ident));
        });
        quote! { #size_assertion }
    }
}

struct LocAttr {
    start: LitInt,
    colon: token::Colon,
    end: LitInt,
}

impl Parse for LocAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            start: input.parse()?,
            colon: input.parse()?,
            end: input.parse()?,
        })
    }
}

impl LocAttr {
    fn into_assertions(self, ident: &Ident, field_ident: &Ident, field_ty: &Type) -> TokenStream {
        let Self { start, end, .. } = self;
        quote! {
            {
                let desc = format!(
                    "{}.{} ({})",
                    ::core::stringify!(#ident),
                    ::core::stringify!(#field_ident),
                    ::core::stringify!(#field_ty),
                );
                let offset = ::memoffset::offset_of!(#ident, #field_ident);
                assert_eq!(#start, offset, "start of {}", desc);
                assert_eq!(#end, ::core::mem::size_of::<#field_ty>() + offset - 1, "end of {}", desc);
            }
        }
    }
}
