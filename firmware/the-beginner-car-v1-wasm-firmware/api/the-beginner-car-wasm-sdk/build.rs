use std::fmt::Write;
use syn::{
    Attribute, Ident, Item, ItemMacro, Result, Token, Type, parse::{Parse, ParseStream, Parser}, punctuated::Punctuated,
};

struct ApiFunction {
    attrs: Vec<Attribute>,
    name: Ident,
    args: Vec<(Ident, Type)>,
    return_type: Option<Type>,
}

impl Parse for ApiFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        input.parse::<Token![fn]>()?;
        let name: Ident = input.parse()?;

        let content;
        syn::parenthesized!(content in input);

        let args = Punctuated::<syn::PatType, Token![,]>::parse_terminated(&content)?
            .into_iter()
            .map(|arg| {
                let ident = match *arg.pat {
                    syn::Pat::Ident(pat) => pat.ident,
                    _ => panic!("Expected a simple argument name"),
                };

                (ident, *arg.ty)
            })
            .collect();

        let return_type = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            attrs,
            name,
            args,
            return_type,
        })
    }
}

fn rust_type_to_c(ty: &Type) -> Result<&'static str> {
    let Type::Path(path) = ty else {
        return Err(syn::Error::new_spanned(ty, "Unsupported C type"));
    };

    let name = path
        .path
        .get_ident()
        .ok_or_else(|| syn::Error::new_spanned(ty, "Unsupported C type"))?
        .to_string();

    match name.as_str() {
        "u8" => Ok("uint8_t"),
        "i8" => Ok("int8_t"),
        "u16" => Ok("uint16_t"),
        "i16" => Ok("int16_t"),
        "u32" => Ok("uint32_t"),
        "i32" => Ok("int32_t"),
        "u64" => Ok("uint64_t"),
        "i64" => Ok("int64_t"),
        "f32" => Ok("float"),
        "f64" => Ok("double"),
        _ => Err(syn::Error::new_spanned(
            ty,
            format!("Unsupported Rust type: {name}"),
        )),
    }
}

fn extract_functions(mac: &ItemMacro) -> Result<Vec<ApiFunction>> {
    let parser = Punctuated::<ApiFunction, Token![;]>::parse_terminated;

    Ok(parser
        .parse2(mac.mac.tokens.clone())?
        .into_iter()
        .collect())
}

fn generate_header(syntax: &syn::File) -> Result<String> {
    let api_macro = syntax.items.iter().find_map(|item| {
        if let Item::Macro(ItemMacro { mac, .. }) = item {
            if mac.path.is_ident("define_wasm_api") {
                return Some(item);
            }
        }

        None
    });

    let Some(Item::Macro(api_macro)) = api_macro else {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Could not find define_wasm_api! invocation",
        ));
    };

    let functions = extract_functions(api_macro)?;

    let mut output = String::new();

    writeln!(output, "#ifndef THE_BEGINNER_H").unwrap();
    writeln!(output, "#define THE_BEGINNER_H\n").unwrap();

    writeln!(output, "#include <stdint.h>\n").unwrap();

    writeln!(output, "#ifdef __cplusplus").unwrap();
    writeln!(output, "extern \"C\" {{").unwrap();
    writeln!(output, "#endif\n").unwrap();

    writeln!(
        output,
        "// ---------------------------------------------------------"
    )
    .unwrap();
    writeln!(
        output,
        "// FFI Imports (Mapped to the \"host\" module in WebAssembly)"
    )
    .unwrap();
    writeln!(
        output,
        "// ---------------------------------------------------------\n"
    )
    .unwrap();

    for function in functions {
        // Preserve Rust documentation comments.
        for attr in &function.attrs {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(meta) = &attr.meta {
                    if let syn::Expr::Lit(expr) = &meta.value {
                        if let syn::Lit::Str(doc) = &expr.lit {
                            writeln!(output, "/** {} */", doc.value().trim()).unwrap();
                        }
                    }
                }
            }
        }

        let name = function.name.to_string();

        let return_type = match &function.return_type {
            Some(ty) => rust_type_to_c(ty)?,
            None => "void",
        };

        let args = function
            .args
            .iter()
            .map(|(arg_name, ty)| {
                // C pointers are represented as i32 in WASM32.
                // The print function receives a pointer to guest memory.
                let c_type = if name == "print" && arg_name == "ptr" {
                    "const char *"
                } else {
                    rust_type_to_c(ty)?
                };

                Ok(format!("{c_type} {arg_name}"))
            })
            .collect::<Result<Vec<_>>>()?;

        let args = if args.is_empty() {
            "void".to_string()
        } else {
            args.join(", ")
        };

        writeln!(
            output,
            "__attribute__((import_module(\"host\"), import_name(\"{name}\")))"
        )
        .unwrap();

        writeln!(output, "extern {return_type} {name}({args});\n").unwrap();
    }

    writeln!(output, "#ifdef __cplusplus").unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output, "#endif\n").unwrap();

    writeln!(output, "#endif // THE_BEGINNER_H").unwrap();

    Ok(output)
}

fn main() {
    let api_path = "src/lib.rs";

    println!("cargo:rerun-if-changed={api_path}");

    let source = std::fs::read_to_string(api_path).expect("Failed to read API definition");

    let syntax = syn::parse_file(&source).expect("Failed to parse API definition");

    let header = generate_header(&syntax).expect("Failed to generate C header");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    let output_path = std::path::Path::new(&out_dir).join("the_beginner.h");

    std::fs::write(&output_path, header).expect("Failed to write C header");

    println!(
        "cargo:warning=Generated C header: {}",
        output_path.display()
    );
}
