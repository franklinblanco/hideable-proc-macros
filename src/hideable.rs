
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, Field, FieldsNamed, FieldsUnnamed, Ident};

pub fn expand_derive_hideable(ident: Ident, data: Data) -> Result<TokenStream, syn::Error> {
    let fields: Vec<Field> = match data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => named.into_iter().collect(),
            syn::Fields::Unnamed(FieldsUnnamed { unnamed: _, .. }) => return Err(syn::Error::new(
                ident.span(),
                "Tuple structs are not yet supported.",
            )),
            syn::Fields::Unit => {
                return Err(syn::Error::new(
                    ident.span(),
                    "Unit structs are not yet supported.",
                ))
            }
        },
        _ => return Err(syn::Error::new(ident.span(), "Only structs are allowed.")),
    };
    let fields_typed: Vec<TokenStream> = fields.iter().map(|field| destructure_field(field)).collect();
    let fields_if_statements: Vec<TokenStream> = fields.iter().map(|field| generate_if_statements(field)).collect();

    let output = quote! {
    impl hideable_types::Hideable for #ident {
        fn hide_fields(&self, attributes: Vec<String>) -> Result<serde_json::Map<String, serde_json::Value>, String> {
            let mut __internal_map = serde_json::Map::new();

            #(#fields_typed) *
            
            for attr in attributes {
                #(#fields_if_statements) *
            }
            Ok(__internal_map)
        }
    }
    };

    Ok(output.into())
    
}

fn destructure_field(field: &Field) -> TokenStream {
    let ident = field.ident.clone().unwrap();
    let field_variable_name = ident;
    let str_field_name = field_variable_name.to_string();
    let type_name = field.ty.to_token_stream(); // get type
    
    let mut field_attrs: TokenStream = quote!{Vec::new()};
    for attr in field.attrs.clone() {
        match attr.path.get_ident() {
            Some(attr_name) => if attr_name.to_string().eq("mark") {
                field_attrs = attr.parse_args().unwrap();
                field_attrs = quote!{Vec::from([#field_attrs])};
            },
            None => {},
        }
    }
    quote!{
        let #field_variable_name: hideable_types::Field<#type_name> = hideable_types::Field { key: #str_field_name.into(), value: self.#field_variable_name.clone(), attributes: #field_attrs };
        __internal_map.insert(#field_variable_name.key.clone(), match serde_json::to_value(#field_variable_name.value) {
            Ok(val) => {val},
            Err(e) => {return Err(e.to_string())},
        });
    }
}
fn generate_if_statements(field: &Field) -> TokenStream {
    let ident = field.ident.clone().unwrap();
    let field_variable_name = ident;
    
    quote!{
        if #field_variable_name.attributes.iter().any(|field_attr| &field_attr.to_string() == &attr) {
            __internal_map.remove(&#field_variable_name.key);
        }
    }
}