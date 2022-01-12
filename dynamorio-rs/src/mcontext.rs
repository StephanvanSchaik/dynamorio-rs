use dynamorio_sys::*;

#[derive(Clone, Copy)]
pub struct MachineContext {
    pub(crate) mcontext: dr_mcontext_t,
}
