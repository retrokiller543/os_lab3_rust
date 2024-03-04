extern crate proc_macro;
use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, FnArg, ItemFn, Pat, Signature};

#[proc_macro_attribute]
pub fn trace_log(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // Extracting the generic parameters and where clause from the original function
    let Signature {
        ident: fn_name,
        inputs,
        output,
        generics,
        ..
    } = input_fn.sig.clone();
    let vis = &input_fn.vis;
    let where_clause = &generics.where_clause;
    let attrs = input_fn.attrs;
    let block = input_fn.block;

    // Preparing to capture function arguments for logging
    let mut args_tokens = Vec::new();
    let mut format_parts = Vec::new();
    let mut arg_names = Vec::new();

    for inp in inputs.iter() {
        match inp {
            FnArg::Typed(pat) => {
                if let Pat::Ident(ref pat_ident) = *pat.pat {
                    let arg_name = &pat_ident.ident;
                    let var_name = format_ident!("{}", arg_name);
                    args_tokens.push(quote! {
                        let #var_name = format!("{:?}", #arg_name);
                    });
                    format_parts.push(format!("{}: {{:?}}", arg_name));
                    arg_names.push(var_name);
                } else {
                    panic!("Unsupported argument pattern");
                }
            }
            FnArg::Receiver(recv) => {
                let self_token = if recv.reference.is_some() {
                    if recv.mutability.is_some() {
                        "self: &mut Self"
                    } else {
                        "self: &Self"
                    }
                } else {
                    "self"
                };
                format_parts.push(self_token.to_string());
            }
        }
    }

    let entering_log = format!("Entering: {}({})", fn_name, format_parts.join(", "));
    let exiting_log = format!("Exiting: {}", fn_name);

    // Wrapping the original function body with log traces
    let wrapped_block = quote! {
        {
            log::trace!(#entering_log, #(#arg_names),*);
            let result = (|| #block)();
            log::trace!(#exiting_log);
            result
        }
    };

    // Reconstructing the function with generics and where clause
    let gen = quote! {
        #(#attrs)*
        #vis fn #fn_name #generics (#inputs) #output #where_clause #wrapped_block
    };

    gen.into()
}

/// Example of [function-like procedural macro][1].
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = quote! {
        #input

        struct Hello;
    };

    tokens.into()
}

/// Example of user-defined [derive mode macro][1]
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-mode-macros
#[proc_macro_derive(MyDerive)]
pub fn my_derive(_input: TokenStream) -> TokenStream {
    let tokens = quote! {
        struct Hello;
    };

    tokens.into()
}

/// Example of user-defined [procedural macro attribute][1].
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros
#[proc_macro_attribute]
pub fn my_attribute(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = quote! {
        #input

        struct Hello;
    };

    tokens.into()
}
