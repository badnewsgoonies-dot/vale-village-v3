use proc_macro::TokenStream;

mod game_entity;
mod game_lifecycle;
mod game_value;

#[proc_macro_attribute]
pub fn game_value(attr: TokenStream, item: TokenStream) -> TokenStream {
    game_value::expand(attr, item)
}

#[proc_macro_attribute]
pub fn game_lifecycle(attr: TokenStream, item: TokenStream) -> TokenStream {
    game_lifecycle::expand(attr, item)
}

#[proc_macro_attribute]
pub fn game_entity(attr: TokenStream, item: TokenStream) -> TokenStream {
    game_entity::expand(attr, item)
}
