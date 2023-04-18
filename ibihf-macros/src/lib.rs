use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(urls))]
struct Opts {
    url_key: String,
    url_key_template: String,
}

#[proc_macro_derive(TemplateUrl, attributes(urls))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let url_key = opts.url_key;
    let url_key_template = opts.url_key_template;
    let answer = quote! {
      const URL_KEY: &'static str = #url_key;
      const URL_KEY_TEMPLATE: &'static str = #url_key_template;
    };

    let output = quote! {
        impl crate::traits::TemplateUrl for #ident<'_> {
            #answer
        }
    };
    output.into()
}
