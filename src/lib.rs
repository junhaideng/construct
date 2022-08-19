use proc_macro::TokenStream;
use proc_macro2::Span;
use std::collections::HashMap;

use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Type};

#[derive(Clone)]
struct FieldAttr {
    setter: bool,       // default true
    getter: bool,       // default true
    ty: Type,           // field type
    name: Ident,        // field name
    setter_name: Ident, // setter name
    getter_name: Ident, // getter name
}

const CONS: &str = "cons";
const RENAME_SETTER: &str = "rename_setter";
const RENAME_GETTER: &str = "rename_getter";
const SETTER: &str = "setter";
const GETTER: &str = "getter";

#[proc_macro_derive(Constructor, attributes(cons))]
pub fn construct(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(s) = data {
        if let Fields::Named(fields) = s.fields {
            let attr: Vec<_> = fields
                .named
                .iter()
                .map(|field| {
                    // filter field attr with `cons`
                    // one filed may have many attrs
                    // if match then convert to self-defined struct `Info`
                    let res: Vec<_> = field
                        .attrs
                        .iter()
                        .filter(|item| {
                            item.path
                                .get_ident()
                                .map(|ident| ident.to_string().eq(CONS))
                                .unwrap_or_default()
                        })
                        .map(|item| {
                            let tokens = item.tokens.to_string();

                            // trim  (  and  )
                            let tokens =
                                tokens.trim_start_matches('(').trim_end_matches(')').trim();

                            let tokens: HashMap<_, _> = tokens
                                .split(',')
                                .map(|item| {
                                    // 根据 = 进行分隔
                                    let kv: Vec<_> = item.splitn(2, '=').collect();
                                    if kv.is_empty() {
                                        panic!("invalid field attr {}", item);
                                    }
                                    if kv.len() == 1 {
                                        return (kv[0].trim(), "");
                                    }

                                    (kv[0].trim(), kv[1].trim())
                                })
                                .collect();

                            let info = FieldAttr {
                                setter: tokens.get(SETTER).map(|v| !v.eq(&"false")).unwrap_or(true),
                                getter: tokens.get(GETTER).map(|v| !v.eq(&"false")).unwrap_or(true),
                                ty: field.ty.clone(),
                                getter_name: tokens
                                    .get(RENAME_GETTER)
                                    .filter(|item| !item.trim().is_empty())
                                    .map(|v| Ident::new(v, Span::call_site()))
                                    .unwrap_or_else(|| field.ident.clone().unwrap()),
                                setter_name: tokens
                                    .get(RENAME_SETTER)
                                    .filter(|item| !item.trim().is_empty())
                                    .map(|v| Ident::new(v, Span::call_site()))
                                    .unwrap_or_else(|| {
                                        Ident::new(
                                            &format!("set_{}", field.ident.to_token_stream()),
                                            Span::call_site(),
                                        )
                                    }),
                                name: field.ident.clone().unwrap(),
                            };
                            info
                        })
                        .collect();

                    // return  default field attr
                    if res.is_empty() {
                        FieldAttr {
                            setter: true,
                            getter: true,
                            ty: field.ty.clone(),
                            name: field.ident.clone().unwrap(),
                            setter_name: Ident::new(
                                &format!("set_{}", field.ident.to_token_stream()),
                                Span::call_site(),
                            ),
                            getter_name: field.ident.clone().unwrap(),
                        }
                    } else {
                        res[0].to_owned()
                    }
                })
                .collect();

            let getter_names: Vec<_> = attr
                .iter()
                .filter(|attr| attr.getter)
                .map(|attr| &attr.name)
                .collect();

            let setter_names: Vec<_> = attr
                .iter()
                .filter(|attr| attr.setter)
                .map(|attr| &attr.name)
                .collect();

            let (getter_idents, getter_types): (Vec<_>, Vec<_>) = attr
                .iter()
                .filter(|item| item.getter)
                .map(|attr| (&attr.getter_name, &attr.ty))
                .unzip();

            let (setter_idents, setter_types): (Vec<_>, Vec<_>) = attr
                .iter()
                .filter(|item| item.setter)
                .map(|attr| (&attr.setter_name, &attr.ty))
                .unzip();

            // construct token stream
            let tokens = quote! {
                impl #ident {
                   #(
                        pub fn #getter_idents(&self) -> &#getter_types {
                            &self.#getter_names
                        }
                    ) *

                   #(
                        pub fn #setter_idents(&mut self, #setter_names: #setter_types)  {
                            self.#setter_names = #setter_names;
                        }
                    ) *

                }
            };
            return tokens.into();
        }
    };
    panic!("Not Support")
}
