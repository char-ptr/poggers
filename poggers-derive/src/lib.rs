use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::crate_name;
// use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn, Ident};

#[proc_macro_attribute]
pub fn create_entry(attr:TokenStream, item:TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let input_name = input.sig.ident.clone();

    let curr_crate = match crate_name("poggers").expect("poggers-derive to be found") {
        proc_macro_crate::FoundCrate::Itself => quote!(crate),
        proc_macro_crate::FoundCrate::Name(x) => {
            let i = Ident::new(&x, Span::call_site());
            quote!(#i)
        },
    };

    let ret = input.sig.output.clone();

    let handle_ret = match ret {
        syn::ReturnType::Default => quote!(),
        syn::ReturnType::Type(_, ty) => {
            if ty.to_token_stream().to_string().contains("Result") {
                quote!{
                    match r {
                        Ok(_) => (),
                        Err(e) => {
                            println!(concat!(stringify!{#input_name}," has errored: {:?}"), e);
                        }
                    }
                }
            } else {
                quote!()
            }
        },
    };

    let alloc_console = quote!{
        unsafe {
            #curr_crate::exports::AllocConsole();
        };
    };
    let free_console = quote!{
        unsafe {
            #curr_crate::exports::FreeConsole();
        };
    };



    TokenStream::from(quote!{
        #input

        #[no_mangle]
        extern "system" fn DllMain(
            h_module : crate::exports::HINSTANCE,
            reason : u32,
            _: *const ::std::ffi::c_void
        ) -> #curr_crate::exports::BOOL {
            match reason {
                crate::exports::DLL_PROCESS_ATTACH => {
                    std::thread::spawn(|| {
                        #alloc_console
                        use ::std::panic;

                        match panic::catch_unwind(||#input_name()) {
                            Err(e) => {
                                println!("`{}` has panicked: {:#?}",stringify!{#input_name}, e);
                            }
                            Ok(r) => {#handle_ret},
                        };

                        #free_console
                    });
                    (true).into()
                }
                _ => (false).into()
            }
        }
    })
}