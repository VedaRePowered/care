use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::ItemFn;

fn care_macro_shared(func: proc_macro::TokenStream, name: &str) -> proc_macro::TokenStream {
    let func = TokenStream::from(func);
    let input: ItemFn = match syn::parse2(func.clone()) {
        Ok(i) => i,
        Err(e) => return token_stream_with_error(func, e),
    };
    let func_name = input.sig.ident.clone();

    std::env::set_var(format!("_CARE_INTERNAL_{name}"), func_name.to_string());

    let result = quote! {
        #input
    };

    result.into()
}

#[proc_macro_attribute]
pub fn care_init(
    _attr: proc_macro::TokenStream,
    func: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    care_macro_shared(func, "INIT")
}

#[proc_macro_attribute]
pub fn care_update(
    _attr: proc_macro::TokenStream,
    func: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    care_macro_shared(func, "UPDATE")
}

#[proc_macro_attribute]
pub fn care_draw(
    _attr: proc_macro::TokenStream,
    func: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    care_macro_shared(func, "DRAW")
}

#[proc_macro]
pub fn care_main(attr: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // TODO: Config
    let attr = TokenStream::from(attr);

    let init_fn = std::env::var(format!("_CARE_INTERNAL_INIT")).ok();
    let update_fn = std::env::var(format!("_CARE_INTERNAL_UPDATE")).ok();
    let draw_fn = std::env::var(format!("_CARE_INTERNAL_DRAW")).ok();

    let init_call = maybe_call_function(init_fn, quote! {});
    let update_call = maybe_call_function(update_fn, quote! {delta_time});
    let draw_call = maybe_call_function(draw_fn, quote! {});

    let result = quote! {
        fn main() {
            #init_call
            loop {
                let delta_time = 0.0 as ::care::math::Fl;
                #update_call
                #draw_call
                ::care::graphics::swap();
            }
        }
    };

    result.into()
}

fn maybe_call_function(fn_name: Option<String>, params: TokenStream) -> TokenStream {
    if let Some(fn_name) = fn_name {
        let fn_ident = Ident::new(&fn_name, Span::call_site());
        quote! {
            #fn_ident(#params);
        }
    } else {
        quote! {}
    }
}

// From tokio (https://github.com/tokio-rs/tokio/blob/tokio-1.35.1/tokio-macros/src/entry.rs#L416)
fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> proc_macro::TokenStream {
    tokens.extend(error.into_compile_error());
    tokens.into()
}
