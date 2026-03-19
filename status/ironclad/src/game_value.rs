use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Error, Fields, Ident, ItemStruct, LitInt, Result, Token, Type};

struct GameValueArgs {
    min: i64,
    max: i64,
}

impl Parse for GameValueArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut min = None;
        let mut max = None;

        while !input.is_empty() {
            let name: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitInt = input.parse()?;
            let parsed = value.base10_parse::<i64>()?;

            match name.to_string().as_str() {
                "min" => min = Some(parsed),
                "max" => max = Some(parsed),
                _ => return Err(Error::new(name.span(), "expected `min` or `max`")),
            }

            if input.is_empty() {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        Ok(Self {
            min: min.ok_or_else(|| input.error("missing `min`"))?,
            max: max.ok_or_else(|| input.error("missing `max`"))?,
        })
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as GameValueArgs);
    let input = parse_macro_input!(item as ItemStruct);

    match expand_inner(args, input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_inner(args: GameValueArgs, input: ItemStruct) -> Result<proc_macro2::TokenStream> {
    if !matches!(input.fields, Fields::Unit) {
        return Err(Error::new_spanned(
            &input,
            "game_value currently expects a unit struct",
        ));
    }

    let vis = input.vis;
    let name = input.ident;
    let repr = value_type();
    let min = args.min;
    let max = args.max;

    Ok(quote! {
        #vis struct #name(#repr);

        impl #name {
            pub fn new(value: #repr) -> Self {
                if value < #min || value > #max {
                    panic!("value out of bounds");
                }

                Self(value)
            }
        }
    })
}

fn value_type() -> Type {
    syn::parse_quote!(i64)
}
