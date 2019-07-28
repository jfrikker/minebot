use darling::ast::{Data, Fields};
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
struct NbtDecodeFieldReceiver {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    #[darling(default)]
    codec: Codec
}

#[derive(Debug, FromVariant)]
#[darling(attributes(nbt))]
struct NbtDecodeVariantReceiver {
    ident: syn::Ident,
    ordinal: i32,
    fields: Fields<NbtDecodeFieldReceiver>
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(enum_any, struct_any))]
pub struct NbtDecodeReceiver {
    pub ident: syn::Ident,
    generics: syn::Generics,
    data: Data<NbtDecodeVariantReceiver, NbtDecodeFieldReceiver>
}

impl quote::ToTokens for NbtDecodeReceiver {
    fn to_tokens(&self, tokens: &mut quote::Tokens) {
        let ident = &self.ident;
        let generics = &self.generics;

        let res = match self.data {
            Data::Enum(ref variants) => {
                let match_arms: Vec<_> = variants.iter()
                    .map(|variant| {
                        let ordinal = variant.ordinal;
                        let variant_ident = variant.ident;
                        let variant_name = quote!(#ident::#variant_ident);
                        let new_val = build_struct(variant_name, &variant.fields);
                        quote! {
                            #ordinal => #new_val
                        }
                    })
                    .collect();


                quote! {
                    impl #generics _nbt::NbtDecode for #ident #generics {
                        fn decode(buf: &mut Bytes) -> Self {
                            let ordinal = _nbt::NbtDecoder::decode(&_nbt::VarNum, buf);
                            match ordinal {
                                #(#match_arms),*,
                                _ => panic!("Unrecognized ordinal {:02X}", ordinal)
                            }
                        }
                    }
                }
            }
            Data::Struct(ref fields) => {
                let name = quote!(#ident);
                let new_val = build_struct(name, fields);

                quote! {
                    impl #generics _nbt::NbtDecode for #ident #generics {
                        fn decode(buf: &mut Bytes) -> Self {
                            #new_val
                        }
                    }
                }
            }
        };

        tokens.append_all(&[res])
    }
}

fn build_struct(name: quote::Tokens, fields: &Fields<NbtDecodeFieldReceiver>) -> quote::Tokens {
    if fields.is_unit() {
        name
    } else if fields.is_tuple() {
        let field_vals: Vec<_> = fields.fields.iter()
            .map(|field| build_decode_field(field))
            .collect();
        quote! {
            #name {
                #(#field_vals),*
            }
        }
    } else {
        let field_vals: Vec<_> = fields.fields.iter()
            .map(|field| {
                let f_name = field.ident.as_ref().unwrap();
                let decoded = build_decode_field(field);
                quote!(#f_name: #decoded)
            })
            .collect();
        quote! {
            #name {
                #(#field_vals),*
            }
        }
    }
}

fn build_decode_field(field: &NbtDecodeFieldReceiver) -> quote::Tokens {
    match field.codec {
        Codec::Default => {
            let ty = &field.ty;
            quote!(<#ty>::decode(buf))
        },
        Codec::VarNum => quote!(_nbt::NbtDecoder::decode(&_nbt::VarNum, buf))
    }
}