use once_cell::sync::OnceCell;

static METHOD_AREA: OnceCell<MethodArea> = OnceCell::new();

pub(in crate::vm::runtime) struct MethodArea {}

pub(crate) fn with_method_area<C, R>(callback: C) -> R
where
    C: FnOnce(&MethodArea) -> R,
{
    let area = METHOD_AREA.get().expect("Failed to get MethodArea");

    callback(&area)
}

