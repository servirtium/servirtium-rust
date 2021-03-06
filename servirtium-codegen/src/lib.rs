use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use quote::quote_spanned;
use std::path::Path;

#[proc_macro_attribute]
pub fn servirtium_record(attrs: TokenStream, item: TokenStream) -> TokenStream {
    servirtium_test(attrs, item, quote! { servirtium::ServirtiumMode::Record }, false)
}

#[proc_macro_attribute]
pub fn servirtium_playback(attrs: TokenStream, item: TokenStream) -> TokenStream {
    servirtium_test(attrs, item, quote! { servirtium::ServirtiumMode::Playback }, false)
}

#[proc_macro_attribute]
pub fn servirtium_record_test(attrs: TokenStream, item: TokenStream) -> TokenStream {
    servirtium_test(attrs, item, quote! { servirtium::ServirtiumMode::Record }, true)
}

#[proc_macro_attribute]
pub fn servirtium_playback_test(attrs: TokenStream, item: TokenStream) -> TokenStream {
    servirtium_test(attrs, item, quote! { servirtium::ServirtiumMode::Playback }, true)
}

fn servirtium_test(
    attrs: TokenStream,
    item: TokenStream,
    enum_variant: proc_macro2::TokenStream,
    with_test_attribute: bool
) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let args = syn::parse_macro_input!(attrs as syn::AttributeArgs);

    let signature = &input.sig;
    let block = &input.block;

    let markdown_name: String;

    if args.len() < 2 {
        return quote! {
            compile_error!("A markdown name and a configuration function should be passed to the macro");
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

    let mut configuration_function = None;
    let mut domain_name = None;
    match &args[1] {
        syn::NestedMeta::Meta(syn::Meta::Path(function_path)) => {
            configuration_function = Some(function_path);
        }
        syn::NestedMeta::Lit(syn::Lit::Str(domain_name_arg)) => {
            domain_name = Some(domain_name_arg.value());
        }
        _ => {
            let error = quote! {
                compile_error!("The second argument should be a configuration function!");
            };

            return error.into();
        }
    };

    let configure = if let Some(configuration_function) = configuration_function {
        quote! { #configuration_function(&mut __servirtium_configuration); }
    } else if let Some(domain_name) = domain_name {
        quote! { __servirtium_configuration.set_domain_name(#domain_name); }
    } else {
        panic!("The configuration function and the domain name are unknown!");
    };

    let test_attribute = if with_test_attribute {
        quote! { #[test] }
    } else {
        quote! {}
    };

    let output = quote! {
        #test_attribute
        #signature {
            let mut __servirtium_configuration = servirtium::ServirtiumConfiguration::new(
                #enum_variant,
                Box::new(servirtium::MarkdownInteractionManager::new(#markdown_name))
            );

            #configure
            servirtium::TestSession::before_test(__servirtium_configuration);

            if let Err(e) = std::panic::catch_unwind(|| {
                #block
            }) {
                if let Err(e) = servirtium::TestSession::after_test() {
                    panic!("Servirtium Error: {}", e);
                }
                std::panic::resume_unwind(e);
            }
            if let Err(e) = servirtium::TestSession::after_test() {
                panic!("Servirtium Error: {}", e);
            }
        }
    };

    TokenStream::from(output)
}

fn validate_markdown_path<P: AsRef<Path>>(
    path: P,
    span: Span,
) -> Result<(), proc_macro2::TokenStream> {
    if !path.as_ref().to_string_lossy().ends_with(".md") {
        return Err(quote_spanned! {span=>
            compile_error!("The path should point to a .md file!");
        });
    }

    Ok(())
}
