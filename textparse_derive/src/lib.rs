use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index, Lit,
    Meta, NestedMeta,
};

fn crate_name() -> TokenStream {
    let textparse =
        proc_macro_crate::crate_name("textparse").expect("textparse is persent in `Cargo.toml`");
    match textparse {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident )
        }
    }
}

#[proc_macro_derive(Span)]
pub fn derive_span_trait(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let textparse = crate_name();
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = add_span_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let start_position = generate_span_start_position_method_body(&input.data);
    let end_position = generate_span_end_position_method_body(&input.data);
    let expanded = quote! {
        impl #impl_generics #textparse::Span for #name #ty_generics #where_clause {
            fn start_position(&self) -> #textparse::Position {
                #start_position
            }
            fn end_position(&self) -> #textparse::Position {
                #end_position
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn add_span_trait_bounds(mut generics: Generics) -> Generics {
    let textparse = crate_name();
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(#textparse::Span));
        }
    }
    generics
}

fn generate_span_start_position_method_body(data: &Data) -> TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let name = &fields.named[0].ident;
                quote! { self.#name.start_position() }
            }
            Fields::Unnamed(_fields) => {
                quote! { self.0.start_position() }
            }
            Fields::Unit => unimplemented!(),
        },
        Data::Enum(data) => {
            let arms = data.variants.iter().map(|variant| {
                let name = &variant.ident;
                if let Fields::Unnamed(fields) = &variant.fields {
                    assert_eq!(fields.unnamed.len(), 1);
                } else {
                    unimplemented!();
                }
                quote_spanned! { variant.span() => Self::#name(x) => x.start_position(), }
            });
            quote! {
                match self {
                    #(#arms)*
                }
            }
        }
        Data::Union(_) => unimplemented!(),
    }
}

fn generate_span_end_position_method_body(data: &Data) -> TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let Some(name) = fields.named.iter().last().map(|f| &f.ident) else {
                    panic!();
                };
                quote! { self.#name.end_position() }
            }
            Fields::Unnamed(fields) => {
                let i = Index::from(fields.unnamed.len() - 1);
                quote! { self.#i.end_position() }
            }
            Fields::Unit => unimplemented!(),
        },
        Data::Enum(data) => {
            let arms = data.variants.iter().map(|variant| {
                let name = &variant.ident;
                if let Fields::Unnamed(fields) = &variant.fields {
                    assert_eq!(fields.unnamed.len(), 1);
                } else {
                    unimplemented!();
                }
                quote_spanned! { variant.span() => Self::#name(x) => x.end_position(), }
            });
            quote! {
                match self {
                    #(#arms)*
                }
            }
        }
        Data::Union(_) => unimplemented!(),
    }
}

#[proc_macro_derive(Parse, attributes(parse))]
pub fn derive_parse_trait(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let textparse = crate_name();
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = add_parse_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let parse = generate_parse_fun_body(&input.data);
    let item_name = if let Some(attrs) = input
        .attrs
        .iter()
        .find(|a| a.path.segments.len() == 1 && a.path.segments[0].ident == "parse")
    {
        let Meta::List(meta_list) = attrs.parse_meta().unwrap() else {
            todo!("{}:{}", module_path!(), line!());
        };
        assert_eq!(meta_list.nested.len(), 1);

        let Some(NestedMeta::Meta(Meta::NameValue(name_value))) = meta_list.nested.first() else {
            todo!("{}:{}", module_path!(), line!());
        };
        assert_eq!(name_value.path.segments.len(), 1);
        assert_eq!(name_value.path.segments[0].ident, "name");

        let Lit::Str(value) = &name_value.lit else {
            todo!("{}:{}", module_path!(), line!());
        };
        quote!(Some(|| #value.to_owned()))
    } else {
        quote!(None)
    };
    let expanded = quote! {
        impl #impl_generics #textparse::Parse for #name #ty_generics #where_clause {
            fn parse(parser: &mut #textparse::Parser) -> Option<Self> {
                #parse
            }

            fn name() -> Option<fn () -> String> {
                #item_name
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn add_parse_trait_bounds(mut generics: Generics) -> Generics {
    let textparse = crate_name();
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(#textparse::Parse));
        }
    }
    generics
}

fn generate_parse_fun_body(data: &Data) -> TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let parse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! { f.span() => #name: parser.parse()? }
                });
                quote! {
                    Some(Self{
                        #(#parse ,)*
                    })
                }
            }
            Fields::Unnamed(fields) => {
                let parse = fields.unnamed.iter().map(|f| {
                    quote_spanned! { f.span() => parser.parse()? }
                });
                quote! {
                    Some(Self(
                        #(#parse ,)*
                    ))
                }
            }
            Fields::Unit => unimplemented!(),
        },
        Data::Enum(data) => {
            let arms = data.variants.iter().map(|variant| {
                let name = &variant.ident;
                if let Fields::Unnamed(fields) = &variant.fields {
                    assert_eq!(fields.unnamed.len(), 1);
                } else {
                    unimplemented!();
                }
                quote_spanned! { variant.span() => if let Some(x) = parser.parse() {
                    return Some(Self::#name(x));
                }}
            });
            quote! {
                #( #arms )*
                None
            }
        }
        Data::Union(_) => unimplemented!(),
    }
}
