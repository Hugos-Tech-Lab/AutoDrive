use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);

    if function.sig.ident != "main" {
        return syn::Error::new_spanned(
            &function.sig.ident,
            "#[auto_drive_sdk::main] must be applied to fn main()",
        )
        .to_compile_error()
        .into();
    }

    if function.sig.inputs.len() != 1 {
        return syn::Error::new_spanned(
            &function.sig.inputs,
            "main must accept one TheBeginnerCar argument",
        )
        .to_compile_error()
        .into();
    }

    if function.sig.asyncness.is_some() {
        return syn::Error::new_spanned(
            &function.sig.asyncness,
            "async main is not supported",
        )
        .to_compile_error()
        .into();
    }

    if !matches!(function.sig.output, syn::ReturnType::Default) {
        return syn::Error::new_spanned(
            &function.sig.output,
            "main must not return a value",
        )
        .to_compile_error()
        .into();
    }

    let helper_name = format_ident!("__auto_drive_user_main");
    function.sig.ident = helper_name.clone();

    quote! {
        #function

        #[unsafe(no_mangle)]
        pub extern "C" fn main() {
            let the_beginner_car =
                ::auto_drive_sdk::hardware::the_beginner_car::__auto_drive_create_beginner_car();

            #helper_name(the_beginner_car);
        }


        #[cfg(not(test))]
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            // TODO add info about the panic
            print("Oh no.. the WASM script panicked.");
            loop {}
        }
    }
    .into()
}