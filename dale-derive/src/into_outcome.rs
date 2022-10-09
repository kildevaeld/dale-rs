use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{GenericParam, TypeParam};

use crate::shared::crate_ident_name;

pub fn implement(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let crate_name = crate_ident_name("dale");

    let mut cloned_gen = ast.generics.clone();

    let next_param = TypeParam::from(syn::Ident::new(
        &format!("{}_ARGS", name.to_string().to_uppercase()),
        Span::call_site(),
    ));

    let next_generic_param = GenericParam::from(next_param.clone());

    cloned_gen.params.push(next_generic_param);

    let gen = &ast.generics;

    let out = quote! {
        impl #cloned_gen #crate_name::IntoOutcome<#next_param> for #name #gen {
            type Success = Self;
            type Failure = core::convert::Infallible;
            fn into_outcome(self) -> #crate_name::Outcome<Self::Success, Self::Failure, #next_param> {
                #crate_name::Outcome::Success(self)
            }
        }
    };

    TokenStream::from(out)
}
