use dynamorio_sys::*;

#[macro_export]
macro_rules! wrap {
    (
        $vis:vis $trait:ident: fn $name:ident($($arg_name:ident : $arg_ty:ty ),*) $(-> $ret:ty)?
    ) => {
        crate::paste! {
            $vis trait [<$trait Handler>] {
                fn $name(&mut self $(, $arg_name : $arg_ty )*) $(-> $ret)?;
            }

            $vis struct [<Registered $trait Handler>]<T: [<$trait Handler>]> {
                _handler: Arc<Mutex<T>>,
                closure: crate::closure::Closure,
                original: dynamorio_sys::app_pc,
            }

            impl<T: [<$trait Handler>]> Drop for [<Registered $trait Handler>]<T> {
                fn drop(&mut self) {
                    unsafe {
                        dynamorio_sys::drwrap_replace(self.original, core::ptr::null_mut(), false as i8);
                    }
                }
            }

            unsafe extern "C" fn [<$name _wrapper>]<T: [<$trait Handler>]>(
                $($arg_name : $arg_ty ,)*
                handler: &Mutex<T>,
            ) $(-> $ret)? {
                if let Ok(mut handler) = handler.lock() {
                    return handler.$name($($arg_name,)*);
                }
            }

            $vis trait [<Register $trait Handler>] {
                unsafe fn [<replace_ $name>]<T: [<$trait Handler>]>(
                    &self,
                    original: dynamorio_sys::app_pc,
                    handler: &Arc<Mutex<T>>,
                ) -> Option<[<Registered $trait Handler>]<T>>;

                fn [<replace_module_$name>]<T: [<$trait Handler>]>(
                    &self,
                    module: &crate::ModuleData,
                    name: &str,
                    handler: &Arc<Mutex<T>>,
                ) -> Option<[<Registered $trait Handler>]<T>>;
            }

            impl [<Register $trait Handler>] for crate::wrap::Wrapper {
                unsafe fn [<replace_ $name>]<T: [<$trait Handler>]>(
                    &self,
                    original: dynamorio_sys::app_pc,
                    handler: &Arc<Mutex<T>>,
                ) -> Option<[<Registered $trait Handler>]<T>> {
                    // Count the number of arguments in the function prototype.
                    #[allow(dead_code, non_camel_case_types)]
                    enum Arguments { $($arg_name,)* Last };
                    let count = Arguments::Last as usize;

                    // Create a closure and attach the handler to the closure, such that the
                    // wrapper function can invoke the handler.
                    let closure = crate::closure::Closure::new(
                        count,
                        unsafe {
                            core::mem::transmute([<$name _wrapper>]::<T>
                                as unsafe extern "C" fn($($arg_ty,)* &Mutex<T>) $(-> $ret)?)
                        },
                        Arc::as_ptr(&handler) as *mut core::ffi::c_void,
                    );

                    // Cast the closure into a function that we can register.
                    let func: extern "C" fn($($arg_ty,)*) $(-> $ret)? = unsafe {
                        core::mem::transmute(closure.code())
                    };

                    unsafe {
                        dynamorio_sys::drwrap_replace(original, func as _, false as i8);
                    }

                    // Return the handler and closure to keep them alive.
                    Some([<Registered $trait Handler>] {
                        _handler: Arc::clone(&handler),
                        closure,
                        original,
                    })
                }

                fn [<replace_module_$name>]<T: [<$trait Handler>]>(
                    &self,
                    module: &crate::ModuleData,
                    name: &str,
                    handler: &Arc<Mutex<T>>,
                ) -> Option<[<Registered $trait Handler>]<T>> {
                    let symbol = match module.get_proc_address(name) {
                        Some(symbol) => symbol,
                        _ => return None,
                    };

                    unsafe {
                        self.[<replace_ $name>](symbol as *mut u8, handler)
                    }
                }
            }
        }
    }
}

pub struct Wrapper;

impl Wrapper {
    pub fn new() -> Self {
        unsafe {
            drwrap_init();
        }

        Self
    }
}
