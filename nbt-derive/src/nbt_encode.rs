use darling::ast::{Data, Fields};
use proc_macro2::Span;
use quote;
use syn;

#[derive(Debug, Clone, Copy, FromMetaItem)]
#[darling(default)]
enum Codec {
    Default,
    #[darling(rename = "varnum")]
    VarNum
}

impl Default for Codec {
    fn default() -> Self {
        Codec::Default
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(nbt))]
struct NbtEncodeFieldReceiver {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    #[darling(default)]
    codec: Codec
}

#[derive(Debug, FromVariant)]
#[darling(attributes(nbt))]
struct NbtEncodeVariantReceiver {
    ident: syn::Ident,
    ordinal: i32,
    fields: Fields<NbtEncodeFieldReceiver>
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(enum_any))]
pub struct NbtEncodeReceiver {
    pub ident: syn::Ident,
    generics: syn::Generics,
    data: Data<NbtEncodeVariantReceiver, ()>
}

impl quote::ToTokens for NbtEncodeReceiver {
    fn to_tokens(&self, tokens: &mut quote::Tokens) {
        let ident = &self.ident;
        let generics = &self.generics;
        let variants = self.data.as_ref()
            .take_enum()
            .unwrap();

        let size_match_arms: Vec<_> = variants.iter()
            .map(|variant| {
                let variant_ident = variant.ident;
                let variant_name = quote!(#ident::#variant_ident);
                build_encoded_size(&variant_name, variant.ordinal, &variant.fields)
            })
            .collect();

        let encode_match_arms: Vec<_> = variants.iter()
            .map(|variant| {
                let variant_ident = variant.ident;
                let variant_name = quote!(#ident::#variant_ident);
                build_encoded(&variant_name, variant.ordinal, &variant.fields)
            })
            .collect();

        let res = quote! {
            impl #generics _nbt::NbtEncode for #ident #generics {
                fn encoded_size(&self) -> usize {
                    match *self {
                        #(#size_match_arms),*
                    }
                }

                fn encode<B: BufMut>(&self, buf: &mut B) {
                    match *self {
                        #(#encode_match_arms),*
                    }
                }
            }
        };

        tokens.append_all(&[res])
    }
}

fn build_encoded_size(name: &quote::Tokens, ordinal: i32, fields: &Fields<NbtEncodeFieldReceiver>) -> quote::Tokens {
    if fields.is_unit() {
        quote!(#name => _nbt::NbtEncoder::encoded_size(&_nbt::VarNum, &#ordinal))
    } else if fields.is_tuple() {
        let field_refs: Vec<_> = fields.fields.iter()
            .enumerate()
            .map(|(i, _)| {
                let f_name = syn::Ident::new(&format!("f{}", i), Span::call_site());
                quote!(ref #f_name)
            })
            .collect();
        let field_vals: Vec<_> = fields.fields.iter()
            .enumerate()
            .map(|(i, field)| {
                let f_name = syn::Ident::new(&format!("f{}", i), Span::call_site());
                let size = build_encode_size_field(&f_name, field);
                quote!(+ #size)
            })
            .collect();
        quote!(#name(#(#field_refs),*) =>  _nbt::NbtEncoder::encoded_size(&_nbt::VarNum, &#ordinal) #(#field_vals)*)
    } else {
        let field_refs: Vec<_> = fields.fields.iter()
            .map(|field| {
                let f_name = &field.ident;
                quote!(ref #f_name)
            })
            .collect();
        let field_vals: Vec<_> = fields.fields.iter()
            .map(|field| {
                let f_name = &field.ident;
                let size = build_encode_size_field(&f_name, field);
                quote!(+ #size)
            })
            .collect();
        quote!(#name{#(#field_refs),*} => _nbt::NbtEncoder::encoded_size(&_nbt::VarNum, &#ordinal) #(#field_vals)*)
    }
}

fn build_encode_size_field<N: quote::ToTokens>(f_name: &N, field: &NbtEncodeFieldReceiver) -> quote::Tokens {
    match field.codec {
        Codec::Default => {
            quote!(#f_name.encoded_size())
        },
        Codec::VarNum => quote!(_nbt::NbtEncoder::encoded_size(&_nbt::VarNum, &#f_name))
    }
}

fn build_encoded(name: &quote::Tokens, ordinal: i32, fields: &Fields<NbtEncodeFieldReceiver>) -> quote::Tokens {
    if fields.is_unit() {
        quote!(#name => _nbt::NbtEncoder::encode(&_nbt::VarNum, &#ordinal, buf))
    } else if fields.is_tuple() {
        let field_refs: Vec<_> = fields.fields.iter()
            .enumerate()
            .map(|(i, _)| {
                let f_name = syn::Ident::new(&format!("f{}", i), Span::call_site());
                quote!(ref #f_name)
            })
            .collect();
        let field_vals: Vec<_> = fields.fields.iter()
            .enumerate()
            .map(|(i, field)| {
                let f_name = syn::Ident::new(&format!("f{}", i), Span::call_site());
                build_encode_field(&f_name, field)
            })
            .collect();
        quote! {
            #name(#(#field_refs),*) => {
                _nbt::NbtEncoder::encode(&_nbt::VarNum, &#ordinal, buf);
                #(#field_vals)*
            }
        }
    } else {
        let field_refs: Vec<_> = fields.fields.iter()
            .map(|field| {
                let f_name = &field.ident;
                quote!(ref #f_name)
            })
            .collect();
        let field_vals: Vec<_> = fields.fields.iter()
            .map(|field| {
                let f_name = &field.ident;
                build_encode_field(&f_name, field)
            })
            .collect();
        quote! {
            #name{#(#field_refs),*} => {
                _nbt::NbtEncoder::encode(&_nbt::VarNum, &#ordinal, buf);
                #(#field_vals)*
            }
        }
    }
}

fn build_encode_field<N: quote::ToTokens>(f_name: &N, field: &NbtEncodeFieldReceiver) -> quote::Tokens {
    match field.codec {
        Codec::Default => {
            quote!(#f_name.encode(buf);)
        },
        Codec::VarNum => quote!(_nbt::NbtEncoder::encode(&_nbt::VarNum, &#f_name, buf);)
    }
}