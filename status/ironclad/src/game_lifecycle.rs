use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Error, Fields, Ident, ItemStruct, Result, Token};

struct GameLifecycleArgs {
    states: Vec<Ident>,
}

impl Parse for GameLifecycleArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut states = vec![input.parse()?];

        while !input.is_empty() {
            input.parse::<Token![->]>()?;
            states.push(input.parse()?);
        }

        if states.len() < 2 {
            return Err(input.error("game_lifecycle expects at least two states"));
        }

        Ok(Self { states })
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as GameLifecycleArgs);
    let input = parse_macro_input!(item as ItemStruct);

    match expand_inner(args, input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_inner(
    args: GameLifecycleArgs,
    input: ItemStruct,
) -> Result<proc_macro2::TokenStream> {
    if !matches!(input.fields, Fields::Unit) {
        return Err(Error::new_spanned(
            &input,
            "game_lifecycle currently expects a unit struct",
        ));
    }

    let ItemStruct { attrs, vis, ident, .. } = input;
    let enum_name = format_ident!("{}State", ident);
    let initial_state = &args.states[0];
    let states = &args.states;
    let transitions = states.windows(2).map(|pair| {
        let current = &pair[0];
        let next = &pair[1];

        quote! {
            impl #ident<#current> {
                pub fn advance(self) -> #ident<#next> {
                    #ident::new()
                }
            }
        }
    });

    Ok(quote! {
        #( #attrs )*
        #vis enum #enum_name {
            #( #states, )*
        }

        #( #vis struct #states; )*

        #vis struct #ident<State> {
            _state: ::std::marker::PhantomData<State>,
        }

        impl<State> #ident<State> {
            pub fn new() -> Self {
                Self {
                    _state: ::std::marker::PhantomData,
                }
            }

            pub fn state(&self) -> #enum_name
            where
                State: LifecycleState,
            {
                State::STATE
            }
        }

        impl<State> ::std::default::Default for #ident<State> {
            fn default() -> Self {
                Self::new()
            }
        }

        #vis trait LifecycleState {
            const STATE: #enum_name;
        }

        #(
            impl LifecycleState for #states {
                const STATE: #enum_name = #enum_name::#states;
            }
        )*

        impl #ident<#initial_state> {
            pub fn begin() -> Self {
                Self::new()
            }
        }

        #( #transitions )*
    })
}

#[cfg(test)]
mod tests {
    use super::{expand_inner, GameLifecycleArgs};
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn expands_unit_struct_into_lifecycle_types() {
        let tokens = expand_inner(
            GameLifecycleArgs {
                states: vec![parse_quote!(Seed), parse_quote!(Sprout), parse_quote!(Harvested)],
            },
            parse_quote!(pub struct Crop;),
        )
        .expect("expansion should succeed");

        let rendered = tokens.to_string();
        assert!(rendered.contains("pub enum CropState"));
        assert!(rendered.contains("pub struct Crop < State >"));
        assert!(rendered.contains("pub trait LifecycleState"));
        assert!(rendered.contains("pub fn advance ( self ) -> Crop < Sprout >"));
        assert!(rendered.contains("pub fn advance ( self ) -> Crop < Harvested >"));
    }

    #[test]
    fn rejects_non_unit_structs() {
        let error = expand_inner(
            GameLifecycleArgs {
                states: vec![parse_quote!(Seed), parse_quote!(Sprout)],
            },
            parse_quote!(pub struct Crop { age: u8 }),
        )
        .expect_err("non-unit struct should fail");

        assert!(error.to_string().contains("unit struct"));
    }

    #[test]
    fn parses_arrow_separated_states() {
        let args: GameLifecycleArgs = syn::parse2(quote!(Seed -> Sprout -> Mature)).unwrap();
        assert_eq!(args.states.len(), 3);
        assert_eq!(args.states[0].to_string(), "Seed");
        assert_eq!(args.states[2].to_string(), "Mature");
    }
}
