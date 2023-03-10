use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{ext::IdentExt, Data, DataStruct, Fields, Meta};

use crate::util::process_field_attrs;

/// Method to derive a [QueryResult](sea_orm::QueryResult)
pub fn expand_derive_from_query_result(ident: Ident, data: Data) -> syn::Result<TokenStream> {
    let fields = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => named.named,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive FromQueryResult on structs");
            })
        }
    };

    let fields = fields.into_iter().map(|field| {
        let name = format_ident!("{}", field.ident.as_ref().unwrap().to_string());
        let name_str = name.unraw().to_string();
        let mut has_default = false;
        let _: Result<(), ()> = process_field_attrs(&field, |meta| {
            if let Meta::Path(path) = meta {
                if let Some(name) = path.get_ident() {
                    if name == "default" {
                        has_default = true;
                    }
                }
            }
            Ok(())
        });
        if has_default {
            quote! {
                #name: row.try_get(pre, #name_str)
                    .or_else(|err| {
                        if err.is_column_not_found_error() {
                            Ok(Default::default())
                        } else {
                            Err(err)
                        }
                    })?
            }
        } else {
            quote! { #name: row.try_get(pre, #name_str)? }
        }
    });

    Ok(quote!(
        #[automatically_derived]
        impl sea_orm::FromQueryResult for #ident {
            fn from_query_result(row: &sea_orm::QueryResult, pre: &str) -> std::result::Result<Self, sea_orm::DbErr> {
                Ok(Self {
                    #(#fields),*
                })
            }
        }
    ))
}
