use proc_macro2::{TokenStream, Ident};
use quote::quote;
use syn::{ItemFn, spanned::Spanned};

fn care_macro_shared(func: proc_macro::TokenStream, name: &str) -> proc_macro::TokenStream {
    let func = TokenStream::from(func);
    let input: ItemFn = match syn::parse2(func.clone()) {
        Ok(i) => i,
        Err(e) => return token_stream_with_error(func, e),
    };
    let func_name = input.sig.ident.clone();
    let export_name = Ident::new(&format!("__care_internal_{name}"), func.span());

    std::env::set_var(format!("_CARE_INTERNAL_{name}"), func_name.to_string());

    let result = quote! {
        #input
        pub use #func_name as #export_name;
    };

    result.into()
}

#[proc_macro_attribute]
pub fn care_init(_attr: proc_macro::TokenStream, func: proc_macro::TokenStream) -> proc_macro::TokenStream {
    care_macro_shared(func, "init")
}

#[proc_macro_attribute]
pub fn care_update(_attr: proc_macro::TokenStream, func: proc_macro::TokenStream) -> proc_macro::TokenStream {
    care_macro_shared(func, "update")
}

#[proc_macro_attribute]
pub fn care_draw(_attr: proc_macro::TokenStream, func: proc_macro::TokenStream) -> proc_macro::TokenStream {
    care_macro_shared(func, "draw")
}

#[proc_macro]
pub fn care_main(attr: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let func = TokenStream::from(func);
    let input: ItemFn = match syn::parse2(func.clone()) {
        Ok(i) => i,
        Err(e) => return token_stream_with_error(func, e),
    };

    std::env::var(format!("_CARE_INTERNAL_{name}"), func_name.to_string());

    let result = quote! {
        #input
        pub use #func_name as #export_name;
    };

    result.into()
}

// From tokio (https://github.com/tokio-rs/tokio/blob/tokio-1.35.1/tokio-macros/src/entry.rs#L416)
fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> proc_macro::TokenStream {
    tokens.extend(error.into_compile_error());
    tokens.into()
}

