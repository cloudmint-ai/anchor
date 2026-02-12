use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ReturnType, parse_macro_input};

// infi loop not supported
#[proc_macro_attribute]
pub fn case(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let is_async = input.sig.asyncness.is_some();
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;

    if let ReturnType::Type(_, _) = input.sig.output {
        return syn::Error::new_spanned(input.sig.output, "test::case should NOT return")
            .to_compile_error()
            .into();
    }

    let mut ignore = quote! {};
    for attr in &input.attrs {
        if attr.path().is_ident("ignore") {
            ignore = quote! { #[ignore] };
            break;
        }
    }

    let result = if is_async {
        quote! {
            #[cfg(not(target_arch = "wasm32"))]
            #[tokio::test(crate = "tokio")]
            #ignore
            async fn #fn_name() -> Result<()> {
                test::init();
                #fn_block
                Ok(())
            }
        }
    } else {
        let body = quote! {
            #ignore
            fn #fn_name() -> Result<()>  {
                test::init();
                #fn_block
                Ok(())
            }
        };
        quote! {
            #[cfg(not(target_arch = "wasm32"))]
            #[test]
            #body
            #[cfg(target_arch = "wasm32")]
            #[wasm_bindgen_test::wasm_bindgen_test]
            #body
        }
    };

    result.into()
}

#[cfg(feature = "runtime")]
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let is_async = input.sig.asyncness.is_some();
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;

    if !is_async {
        return syn::Error::new_spanned(input.sig.output, "main should be async")
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        #[tokio::main]
        async fn #fn_name() -> Result<()> {
            let result: Result<()> = {
                #fn_block
            };
            if let Err(e) = result {
                error!("main error {:?}", e)
            }
            Ok(())
        }
    };

    TokenStream::from(expanded)
}

#[cfg(feature = "api")]
#[proc_macro_derive(Protocol)]
pub fn protocol_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl api::Protocol for #name {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Entity)]
pub fn entity_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl Entity for #name {
            fn _id(&self) -> Id {
                self.id
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Versioned)]
pub fn versioned_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        #[async_trait]
        impl Versioned for #name {
            fn _current_version(&self) -> &Version {
                &self.version
            }
        }
    };

    TokenStream::from(expanded)
}
