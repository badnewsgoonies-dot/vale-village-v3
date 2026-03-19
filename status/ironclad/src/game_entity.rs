use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Error, Field, Fields, Ident, ItemStruct, Result, Token};

struct GameEntityArgs {
    required_fields: Vec<Ident>,
}

impl Parse for GameEntityArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name: Ident = input.parse()?;
        if name != "requires" {
            return Err(Error::new(name.span(), "expected `requires`"));
        }

        input.parse::<Token![=]>()?;

        let content;
        syn::bracketed!(content in input);

        let mut required_fields = Vec::new();
        while !content.is_empty() {
            required_fields.push(content.parse()?);
            if content.is_empty() {
                break;
            }
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        if !input.is_empty() {
            return Err(input.error("unexpected trailing tokens"));
        }

        Ok(Self { required_fields })
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as GameEntityArgs);
    let input = parse_macro_input!(item as ItemStruct);

    match expand_inner(args, input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_inner(args: GameEntityArgs, input: ItemStruct) -> Result<proc_macro2::TokenStream> {
    let vis = input.vis.clone();
    let struct_name = input.ident.clone();
    let builder_name = format_ident!("{}Builder", struct_name);

    let named_fields = match &input.fields {
        Fields::Named(fields) => fields.named.iter().collect::<Vec<_>>(),
        _ => {
            return Err(Error::new_spanned(
                &input,
                "game_entity currently expects a struct with named fields",
            ));
        }
    };

    let required_fields = resolve_required_fields(&args.required_fields, &named_fields)?;
    let state_params = required_fields
        .iter()
        .map(|field| format_ident!("{}State", pascal_case(field)))
        .collect::<Vec<_>>();

    let builder_generics_decl = if state_params.is_empty() {
        quote!()
    } else {
        quote!(<#(#state_params),*>)
    };

    let builder_generics_missing = repeat_marker("Missing", state_params.len());
    let builder_generics_present = repeat_marker("Present", state_params.len());

    let builder_fields = named_fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("named field");
        let field_ty = &field.ty;
        quote!(#field_name: ::core::option::Option<#field_ty>)
    });

    let builder_init_fields = named_fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("named field");
        quote!(#field_name: ::core::option::Option::None)
    });

    let builder_field_names = named_fields
        .iter()
        .map(|field| field.ident.as_ref().expect("named field").clone())
        .collect::<Vec<_>>();

    let optional_setters = named_fields
        .iter()
        .filter(|field| {
            let field_name = field.ident.as_ref().expect("named field");
            !required_fields.iter().any(|required| required == field_name)
        })
        .map(|field| build_optional_setter(field, &builder_name, &builder_field_names, &state_params))
        .collect::<Vec<_>>();

    let required_setters = named_fields
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.as_ref().expect("named field");
            required_fields
                .iter()
                .position(|required| required == field_name)
                .map(|index| build_required_setter(field, index, &builder_name, &builder_field_names, &state_params))
        })
        .collect::<Vec<_>>();

    let build_initializers = named_fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("named field");
        if required_fields.iter().any(|required| required == field_name) {
            quote! {
                #field_name: self.#field_name.expect(concat!(
                    "required field `",
                    stringify!(#field_name),
                    "` missing"
                ))
            }
        } else {
            quote! {
                #field_name: self.#field_name.unwrap_or_default()
            }
        }
    });

    let marker_field = if state_params.is_empty() {
        quote!()
    } else {
        quote!(_marker: ::core::marker::PhantomData<(#(#state_params),*)>,)
    };

    let marker_value = if state_params.is_empty() {
        quote!()
    } else {
        quote!(_marker: ::core::marker::PhantomData,)
    };

    let generic_impl = if state_params.is_empty() {
        quote! {
            impl #builder_name {
                #(#optional_setters)*
            }
        }
    } else {
        quote! {
            impl<#(#state_params),*> #builder_name<#(#state_params),*> {
                #(#optional_setters)*
            }
        }
    };

    let build_impl = if state_params.is_empty() {
        quote! {
            impl #builder_name {
                pub fn build(self) -> #struct_name {
                    #struct_name {
                        #(#build_initializers,)*
                    }
                }
            }
        }
    } else {
        quote! {
            impl #builder_name #builder_generics_present {
                pub fn build(self) -> #struct_name {
                    #struct_name {
                        #(#build_initializers,)*
                    }
                }
            }
        }
    };

    Ok(quote! {
        #input

        #vis struct Missing;
        #vis struct Present;

        #vis struct #builder_name #builder_generics_decl {
            #(#builder_fields,)*
            #marker_field
        }

        impl #struct_name {
            pub fn builder() -> #builder_name #builder_generics_missing {
                #builder_name {
                    #(#builder_init_fields,)*
                    #marker_value
                }
            }
        }

        #generic_impl
        #(#required_setters)*
        #build_impl
    })
}

fn resolve_required_fields(required_fields: &[Ident], fields: &[&Field]) -> Result<Vec<Ident>> {
    let mut resolved = Vec::new();

    for required in required_fields {
        let expected = normalize_required_name(required);
        let field = fields
            .iter()
            .find(|field| {
                field
                    .ident
                    .as_ref()
                    .map(|ident| ident == &expected)
                    .unwrap_or(false)
            })
            .ok_or_else(|| Error::new(required.span(), format!("unknown required field `{required}`")))?;

        resolved.push(field.ident.clone().expect("named field"));
    }

    Ok(resolved)
}

fn build_optional_setter(
    field: &Field,
    builder_name: &Ident,
    builder_field_names: &[Ident],
    state_params: &[Ident],
) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().expect("named field");
    let field_ty = &field.ty;
    let setter_name = format_ident!("set_{}", field_name);
    let assignments = retained_assignments(builder_field_names, field_name);
    let builder_generics = if state_params.is_empty() {
        quote!()
    } else {
        quote!(<#(#state_params),*>)
    };
    let marker_value = if state_params.is_empty() {
        quote!()
    } else {
        quote!(_marker: ::core::marker::PhantomData,)
    };

    quote! {
        pub fn #setter_name(self, value: #field_ty) -> #builder_name #builder_generics {
            #builder_name {
                #(#assignments,)*
                #field_name: ::core::option::Option::Some(value),
                #marker_value
            }
        }
    }
}

fn build_required_setter(
    field: &Field,
    required_index: usize,
    builder_name: &Ident,
    builder_field_names: &[Ident],
    state_params: &[Ident],
) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().expect("named field");
    let field_ty = &field.ty;
    let setter_name = format_ident!("set_{}", field_name);
    let assignments = retained_assignments(builder_field_names, field_name);
    let current_states = state_params
        .iter()
        .enumerate()
        .map(|(index, param)| {
            if index == required_index {
                quote!(Missing)
            } else {
                quote!(#param)
            }
        })
        .collect::<Vec<_>>();
    let next_states = state_params
        .iter()
        .enumerate()
        .map(|(index, param)| {
            if index == required_index {
                quote!(Present)
            } else {
                quote!(#param)
            }
        })
        .collect::<Vec<_>>();

    quote! {
        impl<#(#state_params),*> #builder_name<#(#current_states),*> {
            pub fn #setter_name(self, value: #field_ty) -> #builder_name<#(#next_states),*> {
                #builder_name {
                    #(#assignments,)*
                    #field_name: ::core::option::Option::Some(value),
                    _marker: ::core::marker::PhantomData,
                }
            }
        }
    }
}

fn retained_assignments(builder_field_names: &[Ident], current_field: &Ident) -> Vec<proc_macro2::TokenStream> {
    builder_field_names
        .iter()
        .filter(|name| *name != current_field)
        .map(|name| quote!(#name: self.#name))
        .collect()
}

fn repeat_marker(marker: &str, count: usize) -> proc_macro2::TokenStream {
    if count == 0 {
        quote!()
    } else {
        let marker_ident = format_ident!("{}", marker);
        let markers = std::iter::repeat(marker_ident).take(count).collect::<Vec<_>>();
        quote!(<#(#markers),*>)
    }
}

fn normalize_required_name(required: &Ident) -> Ident {
    let raw = required.to_string();
    let normalized = if raw.chars().any(|ch| ch.is_uppercase()) {
        camel_to_snake(&raw)
    } else {
        raw
    };

    format_ident!("{}", normalized, span = required.span())
}

fn camel_to_snake(input: &str) -> String {
    let mut output = String::new();
    for (index, ch) in input.chars().enumerate() {
        if ch.is_uppercase() {
            if index > 0 {
                output.push('_');
            }
            output.extend(ch.to_lowercase());
        } else {
            output.push(ch);
        }
    }
    output
}

fn pascal_case(ident: &Ident) -> String {
    let mut output = String::new();
    let mut capitalize = true;

    for ch in ident.to_string().chars() {
        if ch == '_' {
            capitalize = true;
            continue;
        }

        if capitalize {
            output.extend(ch.to_uppercase());
            capitalize = false;
        } else {
            output.push(ch);
        }
    }

    output
}
