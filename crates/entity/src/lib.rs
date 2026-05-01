use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, ItemStruct, Meta, parse_macro_input};

/// Collapses Row + Create + Update structs into one canonical declaration.
///
/// Field markers:
/// - `#[model(skip)]` — server-managed; appears in Row only, omitted from Create/Update.
///
/// Validation:
/// - `#[validate(...)]` is stripped from Row, forwarded to Create + Update.
///
/// Generates:
/// - `Name`        — full row, `Serialize` + camelCase
/// - `NameCreate`  — user-supplied subset, `Deserialize` + `Validate` + camelCase
/// - `NameUpdate`  — all-`Option` user subset, `Deserialize` + `Validate` + camelCase
#[proc_macro_attribute]
pub fn model(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let name = &item.ident;
    let vis = &item.vis;
    let create_name = format_ident!("{}Create", name);
    let update_name = format_ident!("{}Update", name);

    let fields = match &item.fields {
        Fields::Named(n) => &n.named,
        _ => {
            return syn::Error::new_spanned(name, "#[model] requires named fields")
                .to_compile_error()
                .into();
        }
    };

    let mut row_fields = Vec::new();
    let mut create_fields = Vec::new();
    let mut update_fields = Vec::new();

    for f in fields {
        let is_skip = f.attrs.iter().any(|a| {
            a.path().is_ident("model")
                && matches!(&a.meta, Meta::List(l) if l.tokens.to_string().trim() == "skip")
        });

        // Row: strip #[model(...)] and #[validate(...)]
        let mut row_f = f.clone();
        row_f
            .attrs
            .retain(|a| !a.path().is_ident("model") && !a.path().is_ident("validate"));
        row_fields.push(row_f);

        if is_skip {
            continue;
        }

        // Create: strip #[model(...)], keep #[validate(...)] and others
        let mut create_f = f.clone();
        create_f.attrs.retain(|a| !a.path().is_ident("model"));
        create_fields.push(create_f);

        // Update: same attrs as Create, but type wrapped in Option
        let ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let v = &f.vis;
        let attrs: Vec<_> = f
            .attrs
            .iter()
            .filter(|a| !a.path().is_ident("model"))
            .collect();
        update_fields.push(quote! {
            #(#attrs)*
            #v #ident: ::core::option::Option<#ty>
        });
    }

    quote! {
        #[derive(::core::fmt::Debug, ::serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        #vis struct #name {
            #(#row_fields,)*
        }

        #[derive(::core::fmt::Debug, ::serde::Deserialize, ::validator::Validate)]
        #[serde(rename_all = "camelCase")]
        #vis struct #create_name {
            #(#create_fields,)*
        }

        #[derive(::core::fmt::Debug, ::serde::Deserialize, ::validator::Validate)]
        #[serde(rename_all = "camelCase")]
        #vis struct #update_name {
            #(#update_fields,)*
        }
    }
    .into()
}
