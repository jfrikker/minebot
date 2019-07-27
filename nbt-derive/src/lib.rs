#[macro_use] extern crate darling;
#[macro_use] extern crate quote;

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

mod nbt_decode;
mod nbt_encode;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::Span;

#[proc_macro_derive(NbtEncode, attributes(nbt))]
pub fn nbt_encode(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let receiver = nbt_encode::NbtEncodeReceiver::from_derive_input(&ast).unwrap();
    let result = wrap_use(&receiver.ident, "nbtencode", &receiver);

    let mut tokens = quote::Tokens::new();
    tokens.append_all(&[result]);
    tokens.into()
}

#[proc_macro_derive(NbtDecode, attributes(nbt))]
pub fn nbt_decode_enum(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let receiver = nbt_decode::NbtDecodeReceiver::from_derive_input(&ast).unwrap();
    let result = wrap_use(&receiver.ident, "nbtdecode", &receiver);

    let mut tokens = quote::Tokens::new();
    tokens.append_all(&[result]);
    tokens.into()
}

fn wrap_use<T: quote::ToTokens>(name: &syn::Ident, ty: &str, content: &T) -> quote::Tokens {
    let dummy_const = syn::Ident::new(&format!("_IMPL_{}_FOR_{}", ty.to_uppercase(), name), Span::call_site());

    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate nbt as _nbt;
            #content
        };
    }
}