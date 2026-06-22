use std::sync::atomic::{AtomicI32, Ordering};

use tracing::trace;

use crate::vm::{
    Result,
    class::CLASSES,
    interpreter,
    method_area::class::State,
    runtime::{
        self,
        method_area::{class::Class, with_method_area},
    },
};

pub(in crate::vm) struct Static {}

static COUNTER: AtomicI32 = AtomicI32::new(0);

impl Static {
    const INIT_METHOD: &'static str = "<clinit>:()V";

    pub(in crate::vm) fn initialise(classname: &str) -> Result<()> {
        let class = CLASSES.get(classname)?;
        Self::initialise_class(&class)
    }

    pub(in crate::vm) fn initialise_class(class: &Class) -> Result<()> {
        let state = class.static_fields_initial_state.lock();

        match state.get_state() {
            State::Initialised => {}
            State::Initialising => trace!("{}: recursively initialising", class.name),
            State::Unitialised => {
                state.set_state(State::Initialising);

                if let Some(parent) = &class.parent {
                    let class = CLASSES.get(parent)?;
                    Self::initialise_class(&class)?;
                }

                let current = COUNTER.fetch_add(1, Ordering::SeqCst);
                trace!("    > {current} initialising {}", class.name);

                if let Ok(method) = class.get_method(Self::INIT_METHOD) {
                    interpreter::execute(method.new_frame()?)?;
                }

                state.set_state(State::Initialised);

                trace!("    < {current} initialised {}", class.name);
            }
        };

        Ok(())
    }
}
