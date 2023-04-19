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

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(table_names))]
struct TableNameOpts {
  table_name: String,
  name_func: String,
  name_table_name: String,
  name_table_name_fk: String,
}

#[proc_macro_derive(NameTableName, attributes(table_names))]
pub fn derive_get(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = TableNameOpts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let table_name = opts.table_name;
    let name_table_name = opts.name_table_name;
    let name_table_name_fk = opts.name_table_name_fk;
    let name_func = opts.name_func;
    let answer = quote! {
        //const TABLE_NAME: &'static str = #table_name;
        const NAME_TABLE_NAME: &'static str = #name_table_name;
        const NAME_TABLE_FK_NAME: &'static str = #name_table_name_fk;
    };

    let query = format!(r#"
SELECT
  {0}.*,
  {1}({0}.id, $2) AS name
FROM {0}
WHERE {0}.id = $1;"#,
      table_name,
      name_func,
    );
    let output = quote! {
        impl NameTableName for #ident {
            #answer
        }
        impl #ident {
          pub async fn get(pool: &sqlx::PgPool, id: i32, lang: i32) -> Result<Option<Self>, sqlx::Error> {
            sqlx::query_as!(
              #ident,
              #query,
              id, lang
            )
            .fetch_optional(pool)
            .await
          }
        }
    };
    output.into()
}
