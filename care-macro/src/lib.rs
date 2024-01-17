use std::collections::HashSet;
use std::io::Write;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Block, Expr, ItemFn, ItemStatic, Stmt};

#[rustfmt::skip]
fn dereference_state_vars(expr: &mut Expr, vars: &HashSet<String>) {
    match expr {
        Expr::Await(syn::ExprAwait { base: expr, .. }) |
        Expr::Cast(syn::ExprCast { expr, .. }) |
        Expr::Field(syn::ExprField { base: expr, .. }) |
        Expr::Group(syn::ExprGroup { expr, .. }) |
        Expr::Let(syn::ExprLet { expr, .. }) |
        Expr::Paren(syn::ExprParen { expr, .. }) |
        Expr::Range(syn::ExprRange { start: Some(expr), end: None, .. } |
                    syn::ExprRange { start: None, end: Some(expr), .. }) |
        Expr::Reference(syn::ExprReference { expr, .. }) |
        Expr::Return(syn::ExprReturn { expr: Some(expr), .. }) |
        Expr::Try(syn::ExprTry { expr, .. }) |
        Expr::Unary(syn::ExprUnary { expr, .. }) |
        Expr::Break(syn::ExprBreak { expr: Some(expr), .. }) |
        Expr::Yield(syn::ExprYield { expr: Some(expr), .. }) => {
            dereference_state_vars(expr, vars);
        }
        Expr::Assign(syn::ExprAssign { left: expr, right: expr2, .. }) |
        Expr::Index(syn::ExprIndex { expr, index: expr2, .. }) |
        Expr::Range(syn::ExprRange { start: Some(expr), end: Some(expr2), .. }) |
        Expr::Binary(syn::ExprBinary { left: expr, right: expr2, .. }) |
        Expr::Repeat(syn::ExprRepeat { expr, len: expr2, .. }) => {
            dereference_state_vars(expr, vars);
            dereference_state_vars(expr2, vars);
        }
        Expr::Array(syn::ExprArray { elems: exprs, .. }) |
        Expr::Tuple(syn::ExprTuple { elems: exprs, .. }) => {
            for expr in exprs {
                dereference_state_vars(expr, vars);
            }
        }
        Expr::Call(syn::ExprCall { func: expr, args: exprs, .. }) |
        Expr::MethodCall(syn::ExprMethodCall { receiver: expr, args: exprs, .. }) => {
            dereference_state_vars(expr, vars);
            for expr in exprs {
                dereference_state_vars(expr, vars);
            }
        }
        Expr::Async(syn::ExprAsync { block: Block { stmts, .. }, ..}) |
        Expr::Block(syn::ExprBlock { block: Block { stmts, .. }, ..}) |
        Expr::Loop(syn::ExprLoop { body: Block { stmts, .. }, .. }) |
        Expr::TryBlock(syn::ExprTryBlock { block: Block { stmts, .. }, .. }) |
        Expr::Unsafe(syn::ExprUnsafe { block: Block { stmts, .. }, ..}) => {
            for stmt in stmts {
                dereference_state_vars_stmt(stmt, vars);
            }
        }
        Expr::ForLoop(syn::ExprForLoop { expr, body: Block { stmts, .. }, .. }) |
        Expr::While(syn::ExprWhile { cond: expr, body: Block { stmts, ..}, .. }) => {
            dereference_state_vars(expr, vars);
            for stmt in stmts {
                dereference_state_vars_stmt(stmt, vars);
            }
        }
        Expr::Match(syn::ExprMatch { expr, arms, .. }) => {
            dereference_state_vars(expr, vars);
            for arm in arms {
                dereference_state_vars(&mut arm.body, vars);
            }
        },
        Expr::Struct(syn::ExprStruct { fields, .. }) => {
            for field in fields {
                dereference_state_vars(&mut field.expr, vars);
            }
        }
        Expr::If(syn::ExprIf { cond: expr, then_branch: block, else_branch, .. }) => {
            dereference_state_vars(expr, vars);
            for stmt in &mut block.stmts {
                dereference_state_vars_stmt(stmt, vars)
            }
            if let Some(else_branch) = else_branch {
                dereference_state_vars(&mut else_branch.1, vars);
            }
        }
        Expr::Path(syn::ExprPath { path: syn::Path { leading_colon: None, segments }, .. }) => {
            if segments.len() == 1 {
                if let Some(seg) = segments.first_mut() {
                    if vars.contains(&seg.ident.to_string()) {
                        *expr = Expr::Paren(syn::ExprParen {
                            attrs: Vec::new(),
                            paren_token: syn::token::Paren(seg.span()),
                            expr: Box::new(Expr::Unary(syn::ExprUnary {
                                attrs: Vec::new(),
                                op: syn::UnOp::Deref(syn::token::Star(seg.span())),
                                expr: Box::new(expr.clone()),
                            })),
                        });
                    }
                }
            }
        }
        _ => {},
    }
}

#[rustfmt::skip]
fn dereference_state_vars_stmt(stmt: &mut Stmt, vars: &HashSet<String>) {
    match stmt {
            syn::Stmt::Local(syn::Local {
                init: Some(init), ..
            }) => dereference_state_vars(&mut init.expr, &vars),
            syn::Stmt::Expr(expr, _) => dereference_state_vars(expr, &vars),
            _ => {}
    }
}

fn care_macro_shared(func: proc_macro::TokenStream, name: &str) -> proc_macro::TokenStream {
    let func = TokenStream::from(func);
    let input: ItemFn = match syn::parse2(func.clone()) {
        Ok(i) => i,
        Err(e) => return token_stream_with_error(func, e),
    };
    let state_params = std::env::var(&"_CARE_INTERNAL_STATE_PARAMS")
        .ok()
        .unwrap_or_default();
    let func_name = input.sig.ident.clone();
    let var_name = format!("_CARE_INTERNAL_{name}");
    if std::env::var(&var_name).is_ok() {
        return func.into();
    }
    std::env::set_var(&var_name, func_name.to_string());

    let state_vars: HashSet<_> = state_params
        .split(",")
        .filter(|s| !s.is_empty())
        .map(|p| {
            p.split_once(':').unwrap().0.trim().to_string()
        })
        .collect();

    let state_params = if input.sig.inputs.is_empty() {
        state_params.trim_start_matches(',')
    } else {
        &state_params
    };
    let new_params: TokenStream = state_params.parse().unwrap();
    let asyncness = input.sig.asyncness;
    let ident = input.sig.ident;
    let generics = input.sig.generics;
    let inputs = input.sig.inputs;
    let output = input.sig.output;
    let mut block = input.block;
    for stmt in &mut block.stmts {
        dereference_state_vars_stmt(stmt, &state_vars);
    }
    let result = quote! {
        #asyncness fn #ident #generics (#inputs #new_params) #output
        #block
    };

    result.into()
}

#[proc_macro_attribute]
pub fn care_state(
    _attr: proc_macro::TokenStream,
    def: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let def = TokenStream::from(def);
    let item: ItemStatic = match syn::parse2::<ItemStatic>(def.clone()) {
        Ok(i) => i,
        Err(e) => return token_stream_with_error(def, e),
    };
    let ident = item.ident.clone();
    let ident_state = Ident::new(&(item.ident.to_string() + "_state"), item.ident.span());
    let ty = item.ty;
    let expr = item.expr;
    std::env::set_var(
        "_CARE_INTERNAL_STATE_DEFS",
        std::env::var("_CARE_INTERNAL_STATE_DEFS")
            .ok()
            .unwrap_or_else(String::new)
            + &quote! { let mut #ident_state: #ty = #expr; }.to_string(),
    );
    std::env::set_var(
        "_CARE_INTERNAL_STATE_PARAMS",
        std::env::var("_CARE_INTERNAL_STATE_PARAMS")
            .ok()
            .unwrap_or_else(String::new)
            + ","
            + &quote! { #ident: &mut #ty }.to_string(),
    );
    std::env::set_var(
        "_CARE_INTERNAL_STATE_ITEMS",
        std::env::var("_CARE_INTERNAL_STATE_ITEMS")
            .ok()
            .unwrap_or_else(String::new)
            + ","
            + &quote! { &mut #ident_state }.to_string(),
    );
    proc_macro::TokenStream::new()
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

    let conf: Expr = match syn::parse2(attr.clone()) {
        Ok(i) => i,
        Err(e) => return token_stream_with_error(attr, e),
    };

    let init_fn = std::env::var(format!("_CARE_INTERNAL_INIT")).ok();
    let update_fn = std::env::var(format!("_CARE_INTERNAL_UPDATE")).ok();
    let draw_fn = std::env::var(format!("_CARE_INTERNAL_DRAW")).ok();

    let state_lets: TokenStream = std::env::var("_CARE_INTERNAL_STATE_DEFS")
        .ok()
        .map(|st| st.parse().unwrap())
        .unwrap_or_else(TokenStream::new);

    let additional_params: TokenStream = std::env::var("_CARE_INTERNAL_STATE_ITEMS")
        .ok()
        .map(|st| st.parse().unwrap())
        .unwrap_or_else(TokenStream::new);
    let additional_params_trim: TokenStream = std::env::var("_CARE_INTERNAL_STATE_ITEMS")
        .ok()
        .map(|st| st.trim_start_matches(',').parse().unwrap())
        .unwrap_or_else(TokenStream::new);

    let init_call = maybe_call_function(init_fn, quote! {app_args #additional_params});
    let update_call = maybe_call_function(update_fn, quote! {delta_time #additional_params});
    let draw_call = maybe_call_function(draw_fn, quote! {#additional_params_trim});

    let result = quote! {
        fn main() {
            let config = { #conf };
            #[cfg(feature = "window")]
            ::care::window::init();
            #[cfg(feature = "window")]
            ::care::window::open(&env!("CARGO_CRATE_NAME"));
            #[cfg(feature = "graphics")]
            ::care::graphics::init();
            #state_lets
            let app_args: Vec<_> = ::std::env::args().collect();
            #init_call
            let mut last_time = ::std::time::Instant::now();
            ::care::event::main_loop(move || {
                let next_time = ::std::time::Instant::now();
                let delta_time = next_time.duration_since(last_time).as_secs_f64() as ::care::math::Fl;
                last_time = next_time;
                #update_call
                #[cfg(feature = "graphics")]
                {
                    #draw_call
                    ::care::graphics::present();
                }
                let _ = ::std::thread::sleep(::std::time::Duration::from_millis(1));
            });
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
