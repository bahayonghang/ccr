use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ReturnType, Type, parse_macro_input, parse_quote};

/// Wraps an async Tauri command in CCR's completion-aware runtime policy.
#[proc_macro_attribute]
pub fn command(_attributes: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);
    let command_name = function.sig.ident.to_string();

    if function.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            &function.sig,
            "CCR runtime-managed Tauri commands must be async",
        )
        .into_compile_error()
        .into();
    }

    if !returns_string_result(&function.sig.output) {
        return syn::Error::new_spanned(
            &function.sig.output,
            "CCR runtime-managed Tauri commands must return Result<T, String>",
        )
        .into_compile_error()
        .into();
    }

    let body = function.block;
    function.block = Box::new(parse_quote!({
        crate::commands::runtime_policy::execute(#command_name, async move #body).await
    }));

    quote! {
        #[tauri::command]
        #function
    }
    .into()
}

fn returns_string_result(output: &ReturnType) -> bool {
    let ReturnType::Type(_, return_type) = output else {
        return false;
    };
    let Type::Path(path) = return_type.as_ref() else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    if segment.ident != "Result" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(Type::Path(error_type))) = arguments.args.last() else {
        return false;
    };
    error_type.path.is_ident("String")
}
