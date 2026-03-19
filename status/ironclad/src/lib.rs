use proc_macro::TokenStream;

mod game_value;

#[proc_macro_attribute]
pub fn game_value(attr: TokenStream, item: TokenStream) -> TokenStream {
    game_value::expand(attr, item)
}

#[proc_macro_attribute]
pub fn game_lifecycle(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn game_entity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
