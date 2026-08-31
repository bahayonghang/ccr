use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ReturnType, Type, parse_macro_input, parse_quote};

/// Wraps an async Tauri command in CCR's completion-aware runtime policy.
#[proc_macro_attribute]
pub fn command(_attributes: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);

    match expand_command(function) {
        Ok(function) => quote! {
            #[tauri::command]
            #function
        }
        .into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_command(mut function: ItemFn) -> syn::Result<ItemFn> {
    if function.sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            &function.sig,
            "CCR runtime-managed Tauri commands must be async",
        ));
    }

    if !returns_string_result(&function.sig.output) {
        return Err(syn::Error::new_spanned(
            &function.sig.output,
            "CCR runtime-managed Tauri commands must return Result<T, String>",
        ));
    }

    let command_name = function.sig.ident.to_string();
    let body = function.block;
    function.block = Box::new(parse_quote!({
        crate::commands::runtime_policy::execute(#command_name, async move #body).await
    }));

    Ok(function)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(function: &ItemFn) -> String {
        quote!(#function).to_string()
    }

    fn expansion_error(function: ItemFn) -> syn::Error {
        match expand_command(function) {
            Ok(_) => panic!("fixture must be rejected"),
            Err(error) => error,
        }
    }

    #[test]
    fn expansion_preserves_attributes_signature_and_wraps_the_body_once() {
        let input: ItemFn = parse_quote! {
            #[doc = "preserved"]
            #[allow(dead_code)]
            pub(crate) async fn fetch<T>(value: T) -> std::result::Result<T, String>
            where
                T: Send + 'static,
            {
                Ok(value)
            }
        };
        let expected: ItemFn = parse_quote! {
            #[doc = "preserved"]
            #[allow(dead_code)]
            pub(crate) async fn fetch<T>(value: T) -> std::result::Result<T, String>
            where
                T: Send + 'static,
            {
                crate::commands::runtime_policy::execute(
                    "fetch",
                    async move { Ok(value) }
                )
                .await
            }
        };

        let expanded = expand_command(input).expect("valid command must expand");

        assert_eq!(tokens(&expanded), tokens(&expected));
    }

    #[test]
    fn accepts_supported_result_path_shapes() {
        for function in [
            parse_quote! {
                async fn direct() -> Result<(), String> { Ok(()) }
            },
            parse_quote! {
                async fn qualified() -> std::result::Result<Vec<String>, String> { Ok(vec![]) }
            },
        ] {
            expand_command(function).expect("supported return shape must expand");
        }
    }

    #[test]
    fn keeps_historical_last_generic_argument_validation() {
        for function in [
            parse_quote! {
                async fn one_type_argument() -> Result<String> { todo!() }
            },
            parse_quote! {
                async fn extra_type_argument() -> Result<(), bool, String> { todo!() }
            },
            parse_quote! {
                async fn lifetime_in_value_position() -> Result<'static, String> { todo!() }
            },
        ] {
            expand_command(function).expect("historically accepted return shape must still expand");
        }
    }

    #[test]
    fn rejects_sync_commands_with_stable_diagnostic() {
        let error = expansion_error(parse_quote! {
            fn sync_command() -> Result<(), String> { Ok(()) }
        });

        assert_eq!(
            error.to_string(),
            "CCR runtime-managed Tauri commands must be async"
        );
        assert_eq!(
            error.into_compile_error().to_string(),
            ":: core :: compile_error ! { \"CCR runtime-managed Tauri commands must be async\" }"
        );
    }

    #[test]
    fn rejects_unsupported_return_shapes_with_stable_diagnostic() {
        let invalid_functions: [ItemFn; 6] = [
            parse_quote! {
                async fn missing_return() {}
            },
            parse_quote! {
                async fn plain_value() -> String { String::new() }
            },
            parse_quote! {
                async fn borrowed_error() -> Result<(), &'static str> { Ok(()) }
            },
            parse_quote! {
                async fn qualified_error() -> Result<(), std::string::String> { Ok(()) }
            },
            parse_quote! {
                async fn result_alias() -> CommandResult<()> { todo!() }
            },
            parse_quote! {
                async fn missing_error_type() -> Result<()> { todo!() }
            },
        ];

        for function in invalid_functions {
            let error = expansion_error(function);
            assert_eq!(
                error.to_string(),
                "CCR runtime-managed Tauri commands must return Result<T, String>"
            );
            assert_eq!(
                error.into_compile_error().to_string(),
                ":: core :: compile_error ! { \"CCR runtime-managed Tauri commands must return Result<T, String>\" }"
            );
        }
    }

    #[test]
    fn async_validation_precedes_return_shape_validation() {
        let error = expansion_error(parse_quote! {
            fn invalid_on_both_axes() {}
        });

        assert_eq!(
            error.to_string(),
            "CCR runtime-managed Tauri commands must be async"
        );
    }
}
