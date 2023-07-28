use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::crate_name;
// use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, Ident, ItemFn, Token};

struct CreateEntryArguments {
    no_console: bool,
    no_thread: bool,
    no_free: bool,
}

impl Parse for CreateEntryArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let argss = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        let mut no_console = false;
        let mut no_thread = false;
        let mut no_free = false;
        for arg in argss {
            match arg.to_string().as_str() {
                "no_console" => {
                    no_console = true;
                }
                "no_thread" => {
                    no_thread = true;
                }
                "no_free" => {
                    no_free = true;
                }
                _ => {}
            }
        }
        Ok(CreateEntryArguments {
            no_console,
            no_thread,
            no_free,
        })
    }
}

/// This macro allows you to define a function which will be called upon dll injection
/// you can get the HMODULE of this dll by simply having a parameter of type `HMODULE`
/// ## Notes
/// On windows, this will automatically allocate a console, if you don't want to do that, use the `no_console` attribute
/// On windows, this will automatically free the console upon dll unload, if you don't want to do that, use the `no_free` attribute
#[proc_macro_attribute]
pub fn create_entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let inputb = input.clone();
    let arg = parse_macro_input!(attr as CreateEntryArguments);
    let input_name = input.sig.ident;
    let has_hmd = !input.sig.inputs.is_empty();

    let curr_crate = match crate_name("poggers").expect("poggers-derive to be found") {
        proc_macro_crate::FoundCrate::Itself => quote!(crate),
        proc_macro_crate::FoundCrate::Name(x) => {
            let i = Ident::new(&x, Span::call_site());
            quote!(#i)
        }
    };

    let ret = input.sig.output;

    let handle_ret = match ret {
        syn::ReturnType::Default => quote!(),
        syn::ReturnType::Type(_, ty) => {
            if ty.to_token_stream().to_string().contains("Result") {
                quote! {
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
        }
    };

    let alloc_console = if arg.no_console {
        quote! {}
    } else {
        quote! {
            unsafe {
                #curr_crate::exports::AllocConsole();
            };
        }
    };
    let free_console = if arg.no_console || arg.no_free {
        quote! {}
    } else {
        quote! {
            unsafe {
                #curr_crate::exports::FreeConsole();
            };
        }
    };
    let call_main = if has_hmd {
        quote! {
            #input_name(h_module)
        }
    } else {
        quote! {
            #input_name()
        }
    };
    let cross_platform = quote! {
        use ::std::panic;

        match panic::catch_unwind(move || #call_main) {
            Err(e) => {
                println!("`{}` has panicked: {:#?}",stringify!{#input_name}, e);
            }
            Ok(r) => {#handle_ret},
        };
    };

    let thread_spawn = if arg.no_thread {
        quote! {#alloc_console;#cross_platform;#free_console}
    } else {
        quote! {
            std::thread::spawn(move || {
                #alloc_console
                #cross_platform
                #free_console
            });
        }
    };

    #[cfg(target_os = "windows")]
    let generated = quote! {
        #[no_mangle]
        extern "system" fn DllMain(
            h_module : #curr_crate::exports::HMODULE,
            reason : u32,
            _: *const ::std::ffi::c_void
        ) -> #curr_crate::exports::BOOL {
            match reason {
                #curr_crate::exports::DLL_PROCESS_ATTACH => {
                    #thread_spawn
                    (true).into()
                }
                _ => (false).into()
            }
        }
    };
    #[cfg(not(target_os = "windows"))]
    let generated = quote! {
        #[#curr_crate::exports::ctor]
        fn lib_init() {
            std::thread::spawn(|| {

                #cross_platform

            });
        }
    };

    TokenStream::from(quote! {
        #inputb

        #generated
    })
}
