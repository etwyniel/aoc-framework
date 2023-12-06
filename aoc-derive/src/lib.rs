use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, AngleBracketedGenericArguments,
    AssocType, Expr, ExprLit, FnArg, GenericArgument, ItemFn, Lit, MetaNameValue, PatType, Path,
    PathArguments, ReturnType, Signature, Token, TraitBound, Type, TypeImplTrait, TypeParamBound,
    TypePath, TypeReference,
};

struct Attributes {
    part: u8,
    example_result: Option<Lit>,
    bench_count: Option<u32>,
}

fn attr_value<'a>(attrs: &'a Punctuated<MetaNameValue, Token![,]>, path: &str) -> Option<&'a Expr> {
    attrs
        .iter()
        .find(|attr| attr.path.is_ident(path))
        .map(|attr| &attr.value)
}

fn required_attr_value<'a>(
    attrs: &'a Punctuated<MetaNameValue, Token![,]>,
    path: &str,
) -> syn::Result<&'a Expr> {
    attr_value(attrs, path)
        .ok_or_else(|| syn::Error::new(attrs.span(), format!("\"{path}\" attribute missing")))
}

fn int_attr(attrs: &Punctuated<MetaNameValue, Token![,]>, path: &str) -> syn::Result<u64> {
    let Expr::Lit(ExprLit {
        lit: Lit::Int(day), ..
    }) = required_attr_value(attrs, path)?
    else {
        return Err(syn::Error::new(
            attrs.span(),
            format!("attribute \"{path}\" must be an integer"),
        ));
    };
    day.base10_parse()
}

fn parse_attrs(attrs: Punctuated<MetaNameValue, Token![,]>) -> syn::Result<Attributes> {
    let part = int_attr(&attrs, "part")? as u8;

    let example_result = attrs
        .iter()
        .find(|attr| attr.path.is_ident("example"))
        .and_then(|attr| match &attr.value {
            Expr::Lit(ExprLit { lit, .. }) => Some(lit.clone()),
            _ => None,
        });

    let bench_count = attr_value(&attrs, "benchmark").and_then(|attr| match &attr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(i), ..
        }) => i.base10_parse().ok(),
        _ => None,
    });

    Ok(Attributes {
        part,
        example_result,
        bench_count,
    })
}

fn get_iterator_item(ty: &Path) -> Option<&Path> {
    let seg = ty.segments.first()?;
    if seg.ident != "Iterator" {
        return None;
    }
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &seg.arguments
    else {
        return None;
    };
    let GenericArgument::AssocType(AssocType {
        ty: Type::Path(TypePath { path, .. }),
        ..
    }) = args.first().as_ref()?
    else {
        return None;
    };
    Some(path)
}

fn get_vec_item(ty: &Path) -> Option<&Path> {
    let seg = ty.segments.first()?;
    if seg.ident != "Vec" {
        return None;
    }
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &seg.arguments
    else {
        return None;
    };
    let GenericArgument::Type(Type::Path(TypePath { path, .. })) = args.first().as_ref()? else {
        return None;
    };
    Some(path)
}

fn convert_bufread(ty: &Type) -> syn::Result<proc_macro2::TokenStream> {
    match ty {
        Type::ImplTrait(TypeImplTrait { bounds, .. }) => {
            if let Some(TypeParamBound::Trait(TraitBound { path, .. })) = bounds.first() {
                if path.is_ident("BufRead") {
                    return Ok(quote!(input));
                }
                if let Some(item) = get_iterator_item(path) {
                    if item.is_ident("String") {
                        return Ok(quote!(input.lines().map(|ln| ln.unwrap())));
                    }
                    if item.is_ident("u8") {
                        return Ok(quote!(input.bytes().map(|b| b.unwrap())));
                    }
                    if item.segments.last().unwrap().ident == "Vec" {
                        return Ok(quote!(input.split(b'\n').map(|ln| ln.unwrap())));
                    }
                }
            }
        }
        Type::Path(TypePath { path, .. }) => {
            if let Some(item) = get_vec_item(path) {
                if item.is_ident("String") {
                    return Ok(quote!(input
                        .lines()
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap()));
                }
                if item.is_ident("u8") {
                    return Ok(quote!(input
                        .bytes()
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap()));
                }
            }
        }
        Type::Reference(TypeReference { elem, .. }) => match elem.as_ref() {
            Type::Path(TypePath { path, .. }) if path.is_ident("str") => {
                return Ok(quote!(&{
                    let mut out = String::new();
                    input.read_to_string(&mut out).unwrap();
                    out
                }))
            }
            _ => (),
        },
        _ => (),
    }
    Err(syn::Error::new(
        ty.span(),
        "Supported types are Vec<String>, Vec<u8>, impl Iterator<String>, impl Iterator<Item = u8> and BufRead",
    ))
}

fn returns_result(sig: &Signature) -> bool {
    let ReturnType::Type(_, ty) = &sig.output else {
        return false;
    };
    let Type::Path(TypePath { path, .. }) = ty.as_ref() else {
        return false;
    };
    path.segments.last().unwrap().ident == "Result"
}

fn impl_part(function: ItemFn, attrs: Attributes) -> syn::Result<proc_macro2::TokenStream> {
    let sig = &function.sig;
    let fn_ident = &sig.ident;
    let Attributes {
        part,
        example_result,
        bench_count,
    } = attrs;
    let example_const = example_result
        .map(|res| match res {
            Lit::Str(s) => {
                quote!(Some(aoc_framework::StrConst(#s)))
            }
            Lit::Int(i) => quote!(Some(aoc_framework::Num(#i))),
            _ => quote!(None),
        })
        .map(|val| quote!(const EXAMPLE_RESULT: Option<aoc_framework::Answer> = #val;));
    let Some(FnArg::Typed(PatType { ty, .. })) = sig.inputs.first() else {
        panic!()
    };
    let conversion = convert_bufread(ty)?;
    let result_conv = if returns_result(sig) {
        quote!(res.map(|res| res.into()))
    } else {
        quote!(Ok(res.into()))
    };
    let bench = if let Some(count) = bench_count {
        quote!(
        fn bench(mut input: impl std::io::BufRead) -> Option<std::time::Duration> {
            let converted = #conversion;
            let start = std::time::Instant::now();
            for _ in 0..#count {
                _ = #fn_ident(&converted);
            }
            Some(start.elapsed() / #count)
        }
        )
    } else {
        quote!()
    };
    Ok(quote!(
        #[doc(hidden)]
        #[allow(nonstandard_style)]
        pub struct #fn_ident {}
        impl aoc_framework::Part for #fn_ident {
        const N: u8 = #part;
        #example_const

        fn run(mut input: impl std::io::BufRead) -> anyhow::Result<aoc_framework::Answer> {
            let res = #fn_ident(#conversion);
            #result_conv
        }

        #bench
    }
    #function
    ))
}

#[proc_macro_attribute]
pub fn aoc(attr: TokenStream, input: TokenStream) -> TokenStream {
    let function = parse_macro_input!(input as ItemFn);
    let attr_list =
        parse_macro_input!(attr with Punctuated<MetaNameValue, Token![,]>::parse_terminated);
    let attrs = match parse_attrs(attr_list) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };
    impl_part(function, attrs)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
