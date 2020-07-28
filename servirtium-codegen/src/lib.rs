use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use quote::quote_spanned;
use std::path::Path;

#[proc_macro_attribute]
pub fn servirtium_record_test(attrs: TokenStream, item: TokenStream) -> TokenStream {
    servirtium_test(attrs, item, quote! { servirtium::ServirtiumMode::Record })
}

#[proc_macro_attribute]
pub fn servirtium_playback_test(attrs: TokenStream, item: TokenStream) -> TokenStream {
    servirtium_test(attrs, item, quote! { servirtium::ServirtiumMode::Playback })
}

fn servirtium_test(
    attrs: TokenStream,
    item: TokenStream,
    enum_variant: proc_macro2::TokenStream,
) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let args = syn::parse_macro_input!(attrs as syn::AttributeArgs);

    let signature = &input.sig;
    let block = &input.block;

    let markdown_name: String;

    if args.len() < 2 {
        return quote! {
            compile_error!("A markdown name, a configuration function should be passed to the macro");
        }
        .into();
    }

    if let syn::NestedMeta::Lit(syn::Lit::Str(parsed_markdown_name)) = &args[0] {
        markdown_name = parsed_markdown_name.value();
        if let Err(stream) = validate_markdown_path(&markdown_name, parsed_markdown_name.span()) {
            return stream.into();
        }
    } else {
        return quote! {
            compile_error!("The first argument should be a string literal!");
        }
        .into();
    }

    let configuration_function;
    if let syn::NestedMeta::Meta(syn::Meta::Path(function_path)) = &args[1] {
        configuration_function = function_path;
    } else {
        let error = quote! {
            compile_error!("The second argument should be a configuration function!");
        };

        return error.into();
    }

    let output = quote! {
        #[test]
        #signature {
            let mut __servirtium_configuration = Default::default();
            #configuration_function(&mut __servirtium_configuration);
            let __servirtium_server_lock = servirtium::prepare_for_test(#enum_variant, #markdown_name,
                &__servirtium_configuration);

            if let Err(e) = std::panic::catch_unwind(|| {
                #block
            }) {
                drop(__servirtium_server_lock);
                std::panic::resume_unwind(e);
            }
        }
    };

    TokenStream::from(output)
}

fn validate_markdown_path<P: AsRef<Path>>(
    path: P,
    span: Span,
) -> Result<(), proc_macro2::TokenStream> {
    let mut absoulte_path = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            let compile_error_message = format!("Couldn't get the current directory: {}", e);
            return Err(quote! {span=>
                compile_error!(#compile_error_message);
            });
        }
    };

    absoulte_path.push(path.as_ref());

    let parent = match absoulte_path.parent() {
        Some(parent) => parent,
        None => {
            return Err(quote! {span=>
                compile_error!("The markdown path should point to a file!");
            });
        }
    };

    if !parent.exists() {
        return Err(quote_spanned! {span=>
            compile_error!("The directory doesn't exist");
        });
    }

    Ok(())
}
