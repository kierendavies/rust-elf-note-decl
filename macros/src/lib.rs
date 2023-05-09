use decl_model::{note::SECTION, Data, VERSION};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, LitInt, LitStr, Token,
};

struct DataArgs(Data);

impl Parse for DataArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let an_int = input.parse::<LitInt>()?.base10_parse()?;

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }
        // If input is empty, then parse_terminated will immediately accept and some_strings will be an empty Vec.
        let some_strings = input
            .parse_terminated(<LitStr as Parse>::parse, Token![,])?
            .iter()
            .map(LitStr::value)
            .collect();

        Ok(DataArgs(Data {
            an_int,
            some_strings,
        }))
    }
}

#[proc_macro]
pub fn data(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DataArgs(data) = parse_macro_input!(input as DataArgs);

    let version_note = note(
        Ident::new("DECL_VERSION", Span::call_site()),
        quote!(::decl::model::note::NoteType::Version),
        VERSION.as_bytes(),
    );

    let data_json = serde_json::to_vec(&data).unwrap();

    let data_note = note(
        Ident::new("DECL_DATA", Span::call_site()),
        quote!(::decl::model::note::NoteType::Data),
        &data_json,
    );

    let tokens = quote! {
        #version_note
        #data_note
    };

    tokens.into()
}

fn note(ident: Ident, note_type: TokenStream, desc: &[u8]) -> TokenStream {
    let desc_literal = Literal::byte_string(desc);
    let desc_size = desc.len();

    quote! {
        #[link_section = #SECTION]
        #[used]
        static #ident: ::decl::model::note::Note<[u8; #desc_size]> = ::decl::model::note::Note::new(#note_type, *#desc_literal);
    }
}
