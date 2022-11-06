use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics};

#[proc_macro_derive(Span)]
pub fn derive_span_trait(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = add_span_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let start_position = generate_span_start_position_method_body(&input.data);
    let end_position = generate_span_end_position_method_body(&input.data);
    let expanded = quote! {
        impl #impl_generics ::textparse::Span for #name #ty_generics #where_clause {
            fn start_position(&self) -> ::textparse::Position {
                #start_position
            }
            fn end_position(&self) -> ::textparse::Position {
                #end_position
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn add_span_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(::textparse::Span));
        }
    }
    generics
}

fn generate_span_start_position_method_body(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                if let Some(field) = fields.named.first() {
                    let name = &field.ident;
                    quote! { self.#name.start_position() }
                } else {
                    unimplemented!()
                }
            }
            Fields::Unnamed(ref fields) => {
                assert_eq!(fields.unnamed.len(), 1);
                quote! { self.0.start_position() }
            }
            Fields::Unit => unimplemented!(),
        },
        Data::Enum(ref data) => {
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
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                if let Some(field) = fields.named.last() {
                    let name = &field.ident;
                    quote! { self.#name.end_position() }
                } else {
                    unimplemented!()
                }
            }
            Fields::Unnamed(ref fields) => {
                assert_eq!(fields.unnamed.len(), 1);
                quote! { self.0.end_position() }
            }
            Fields::Unit => unimplemented!(),
        },
        Data::Enum(ref data) => {
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

#[proc_macro_derive(Parse)]
pub fn derive_parse_trait(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = add_parse_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let parse = generate_parse_fun_body(&input.data);
    let expanded = quote! {
        impl #impl_generics ::textparse::Parse for #name #ty_generics #where_clause {
            fn parse(parser: &mut ::textparse::Parser) -> ::textparse::ParseResult<Self> {
                use orfail::OrFail;
                #parse
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn add_parse_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(::textparse::Parse));
        }
    }
    generics
}

fn generate_parse_fun_body(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let parse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! { f.span() => #name: parser.parse()? }
                });
                quote! {
                    Ok(Self{
                        #(#parse ,)*
                    })
                }
            }
            Fields::Unnamed(ref fields) => {
                assert_eq!(fields.unnamed.len(), 1);
                quote! { parser.parse().map(Self) }
            }
            Fields::Unit => unimplemented!(),
        },
        Data::Enum(ref data) => {
            let arms = data.variants.iter().map(|variant| {
                let name = &variant.ident;
                if let Fields::Unnamed(fields) = &variant.fields {
                    assert_eq!(fields.unnamed.len(), 1);
                } else {
                    unimplemented!();
                }
                quote_spanned! { variant.span() => if let Ok(x) = parser.parse() {
                    return Ok(Self::#name(x));
                }}
            });
            quote! {
                #( #arms )*
                Err(::textparse::ParseError::Failed)
            }
        }
        Data::Union(_) => unimplemented!(),
    }
}
