use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct ConfigField {
    name: Ident,
    ty: Type,
    default: Expr,
}

struct ConfigInput {
    name: Ident,
    fields: Vec<ConfigField>,
}

impl Parse for ConfigField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![=]>()?;
        let default: Expr = input.parse()?;
        Ok(ConfigField { name, ty, default })
    }
}

impl Parse for ConfigInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let content;
        syn::braced!(content in input);
        let fields = content
            .parse_terminated(ConfigField::parse, Token![,])?
            .into_iter()
            .collect();
        Ok(ConfigInput { name, fields })
    }
}

/// Defines a configuration struct — see the [`gifnoc` crate docs](https://docs.rs/gifnoc) for full usage.
#[proc_macro]
pub fn config(input: TokenStream) -> TokenStream {
    let ConfigInput { name, fields } = parse_macro_input!(input as ConfigInput);

    let field_defs = fields.iter().map(|f| {
        let fname = &f.name;
        let ftype = &f.ty;
        quote! { pub #fname: #ftype }
    });

    let field_defaults = fields.iter().map(|f| {
        let fname = &f.name;
        let fdefault = &f.default;
        quote! { #fname: #fdefault.into() }
    });

    let expanded = quote! {
        #[derive(serde::Deserialize, serde::Serialize)]
        pub struct #name {
            #(#field_defs),*
        }

        impl Default for #name {
            fn default() -> Self {
                #name {
                    #(#field_defaults),*
                }
            }
        }

        impl ::gifnoc::Configurable for #name {}
    };

    TokenStream::from(expanded)
}
