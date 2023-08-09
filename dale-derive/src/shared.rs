use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};

pub fn crate_ident_name(name: &str) -> syn::Ident {
    let found_crate = crate_name(name).unwrap_or_else(|_| panic!("{} is present in `Cargo.toml`", name));

    let name = name.replace('-', "_");

    match found_crate {
        FoundCrate::Itself => syn::Ident::new(&name, Span::call_site()),
        FoundCrate::Name(name) => {
            
            syn::Ident::new(&name, Span::call_site())
        }
    }
}
