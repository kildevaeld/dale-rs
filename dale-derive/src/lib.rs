mod into_outcome;
mod shared;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(IntoOutcome)]
pub fn into_outcome(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = parse_macro_input!(input as DeriveInput);

    // Build the impl
    let gen = into_outcome::implement(&ast);

    // Return the generated impl
    TokenStream::from(gen)
}
