use dynamorio_sys::*;

#[derive(Clone, Copy)]
pub struct MachineContext {
    pub(crate) _mcontext: dr_mcontext_t,
}
