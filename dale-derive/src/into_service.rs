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
        impl #cloned_gen #crate_name::IntoService<#next_param> for #name #gen {
            type Error = ::core::convert::Infallible;
            type Service = Self;
            fn into_service(self) -> Result<Self::Service, Self::Error> {
                Ok(self)
            }
        }
    };

    out
}
