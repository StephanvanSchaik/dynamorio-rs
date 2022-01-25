use dynamorio_sys::*;

#[macro_export]
macro_rules! wrap {
    (
        $vis:vis $trait:ident: fn $name:ident($($arg_name:ident : $arg_ty:ty ),*) $(-> $ret:ty, $default:expr)?
    ) => {
        crate::paste! {
            $vis trait [<$trait Handler>] {
                fn $name(&mut self $(, $arg_name : $arg_ty )*) $(-> $ret)?;
            }

            $vis struct [<Registered $trait Handler>]<T: [<$trait Handler>]> {
                _handler: Arc<Mutex<T>>,
                original: dynamorio_sys::app_pc,
            }

            unsafe impl<T: [<$trait Handler>]> Send for [<Registered $trait Handler>]<T> {}
            unsafe impl<T: [<$trait Handler>]> Sync for [<Registered $trait Handler>]<T> {}

            impl<T: [<$trait Handler>]> Drop for [<Registered $trait Handler>]<T> {
                fn drop(&mut self) {
                    /*unsafe {
                        dynamorio_sys::drwrap_replace_native(
                            self.original, core::ptr::null_mut(), false as i8);
                    }*/
                }
            }

            unsafe extern "C" fn [<$name _wrapper>]<T: [<$trait Handler>]>(
                $($arg_name : $arg_ty ,)*
                //handler: &Mutex<T>,
            ) $(-> $ret)? {
                let context = Context::current();
                let handler: &Mutex<T> = core::mem::transmute(
                    context.read_saved_register(dr_spill_slot_t::SPILL_SLOT_2)
                );

                if let Ok(mut handler) = handler.lock() {
                    let result = handler.$name($($arg_name,)*);

                    let context = dynamorio_sys::dr_get_current_drcontext();
                    dynamorio_sys::drwrap_replace_native_fini(context);

                    return result;
                }

                let context = dynamorio_sys::dr_get_current_drcontext();
                dynamorio_sys::drwrap_replace_native_fini(context);
                $($default)?
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
                    let result = unsafe {
                        dynamorio_sys::drwrap_replace_native(
                            original,
                            core::mem::transmute([<$name _wrapper>]::<T>
                                as unsafe extern "C" fn($($arg_ty,)*) $(-> $ret)?),
                            true as i8,
                            0,
                            Arc::as_ptr(&handler) as *mut core::ffi::c_void,
                            false as i8) != 0
                    };

                    if !result {
                        return None;
                    }

                    // Return the handler to keep it alive.
                    Some([<Registered $trait Handler>] {
                        _handler: Arc::clone(&handler),
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
