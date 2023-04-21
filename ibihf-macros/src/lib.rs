use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Field, Attribute, Ident};

fn matching_attr_map(attr: &Attribute, attr_name: &str) -> bool {
  if let Ok(syn::Meta::List(meta_list)) = attr.parse_meta() {
      return meta_list.path.is_ident("table_names")
          && meta_list.nested.iter().any(|nested_meta| {
              if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = nested_meta {
                  path.is_ident(attr_name)
              } else {
                  false
              }
          });
  }
  false
}

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

fn get_map_filter(field: &Field) -> Option<String> {
  let name = &field.ident.as_ref().unwrap();
  if field.attrs.iter().any(|attr| attr.path.is_ident("get")) {
    Some(name.to_string())
  } else {
    None
  }
}
fn get_many_map_filter(field: &Field) -> Option<String> {
  let name = &field.ident.as_ref().unwrap();
  if field.attrs.iter().any(|attr| matching_attr_map(attr, "get_many")) {
    Some(name.to_string())
  } else {
    None
  }
}

#[proc_macro_derive(NameTableName, attributes(table_names))]
pub fn derive_get(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = TableNameOpts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let fields = match input.data {
        Data::Struct(ref data) => &data.fields,
        _ => panic!("MyDerive only supports structs"),
    };
    
    let by_many_names: Vec<String> = fields.iter().filter_map(get_many_map_filter).collect();
    let by_many_funcs: Vec<TokenStream2> = by_many_names.iter()
      .map(|name| {
        let query = format!(r#"
SELECT
  {0}.*,
  {1}(id, $2) AS name
FROM {0}
WHERE {name} = $1;
"#, opts.table_name, opts.name_func);
        let method = Ident::new(&format!("by_{}", name), Span::call_site());
        let id_name = Ident::new(&format!("{}_id", name), Span::call_site());
        quote! {
          pub async fn #method(pool: &sqlx::PgPool, #id_name: i32, lang: i32) -> Result<Vec<Self>, sqlx::Error> {
            sqlx::query_as!(
              Self,
              #query,
              #id_name, lang
            )
            .fetch_all(pool)
            .await
          }
        }
      }.into())
      .collect();
      

    let table_name = opts.table_name;
    let name_table_name = opts.name_table_name;
    let name_table_name_fk = opts.name_table_name_fk;
    let name_func = opts.name_func;
    let answer = quote! {
        //const TABLE_NAME: &'static str = #table_name;
        const NAME_TABLE_NAME: &'static str = #name_table_name;
        const NAME_TABLE_FK_NAME: &'static str = #name_table_name_fk;
    };

    let get_query = format!(r#"
SELECT
  {0}.*,
  {1}({0}.id, $2) AS name
FROM {0}
WHERE {0}.id = $1;"#,
      table_name,
      name_func,
    );
    let all_query = format!(r#"
SELECT
  {0}.*,
  {1}({0}.id, $1) AS name
FROM {0}"#,
      table_name,
      name_func,
    );
    let output = quote! {
        impl NameTableName for #ident {
            #answer
        }
        impl #ident {
          #(#by_many_funcs)*
          pub async fn all(pool: &sqlx::PgPool, lang: i32) -> Result<Vec<Self>, sqlx::Error> {
            sqlx::query_as!(
              #ident,
              #all_query,
              lang
            )
            .fetch_all(pool)
            .await
          }
          pub async fn get(pool: &sqlx::PgPool, id: i32, lang: i32) -> Result<Option<Self>, sqlx::Error> {
            sqlx::query_as!(
              #ident,
              #get_query,
              id, lang
            )
            .fetch_optional(pool)
            .await
          }
        }
    };
    output.into()
}
