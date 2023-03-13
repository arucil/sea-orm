use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{ext::IdentExt, Data, DataStruct, Fields, Lit, Meta};

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

    let fields = fields
        .into_iter()
        .map(|field| {
            let name = format_ident!("{}", field.ident.as_ref().unwrap().to_string());
            let mut name_str = name.unraw().to_string();
            let mut has_default = false;
            process_field_attrs(&field, |meta| {
                match meta {
                    Meta::Path(path) => {
                        if let Some(name) = path.get_ident() {
                            if name == "default" {
                                has_default = true;
                            }
                        }
                    }
                    Meta::NameValue(nv) => {
                        if let Some(name) = nv.path.get_ident() {
                            if name == "column_name" {
                                if let Lit::Str(name) = &nv.lit {
                                    name_str = name.value();
                                } else {
                                    return Err(syn::Error::new(
                                        nv.lit.span(),
                                        format!("Invalid column_name {:?}", nv.lit),
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }
                Ok(())
            })?;
            if has_default {
                Ok(quote! {
                    #name: row.try_get(pre, #name_str)
                        .or_else(|err| {
                            if err.is_column_not_found_error() {
                                Ok(Default::default())
                            } else {
                                Err(err)
                            }
                        })?
                })
            } else {
                Ok(quote! { #name: row.try_get(pre, #name_str)? })
            }
        })
        .collect::<syn::Result<Vec<_>>>()?;

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
