use darling::{export::NestedMeta, FromMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Expr, Field, ItemStruct};

use crate::{queries::upsert::upsert_impl, token_stream_with_error};

/// Attribute expand
/// Just adds the dervie macro to the struct.
pub fn expand(input: TokenStream) -> TokenStream {
    let input: ItemStruct = match syn::parse2(input.clone()) {
        Ok(it) => it,
        Err(e) => return token_stream_with_error(input, e),
    };

    let input_clone = input.clone();
    let pks = input_clone
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("pk")))
        .collect::<Vec<_>>();

    if pks.is_empty() {
        return token_stream_with_error(
            input.clone().into_token_stream(),
            syn::Error::new_spanned(
                input.clone().into_token_stream(),
                "Entity can only be derived for structs with at least one #[pk] field.",
            ),
        );
    }

    entity_impl(&input, &pks)
}

/// EntityExt implementation
fn entity_impl(input: &ItemStruct, pks: &Vec<&Field>) -> TokenStream {
    let name = &input.ident;

    let keys = input.fields.iter().map(get_field_name).collect::<Vec<_>>();
    let pks = pks.iter().map(|x| get_field_name(x)).collect::<Vec<_>>();

    quote! {
        impl scyllax::EntityExt<#name> for #name {
            fn keys() -> Vec<String> {
                vec![#(#keys.to_string()),*]
            }

            fn pks() -> Vec<String> {
                vec![#(#pks.to_string()),*]
            }
        }
    }
}

/// This is used to get the name of a field, taking into account the `#[rename]` attribute.  
///
/// Rename is usually used to support camelCase keys, which need to be wrapped
/// in quotes or scylla will snake_ify it.
pub fn get_field_name(field: &Field) -> String {
    let rename = field.attrs.iter().find(|a| a.path().is_ident("rename"));
    if let Some(rename) = rename {
        let expr = rename.parse_args::<Expr>().expect("Expected an expression");
        if let Expr::Lit(lit) = expr {
            if let syn::Lit::Str(s) = lit.lit {
                return format!(r##""{}""##, s.value());
            }
        }
    }

    field
        .ident
        .as_ref()
        .expect("Expected field to have a name")
        .to_string()
}
