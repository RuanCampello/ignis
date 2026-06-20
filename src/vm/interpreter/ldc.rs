use parking_lot::RwLock;
use std::collections::HashMap;

use crate::vm::VmError;

/// ldc (load constant) resolution manager.
///
/// This is used to lazy resolution and caching of
/// constant pool entries used by the JVM bytecode `ldc` and `ldc_w` instructions
///
/// The `ldc` and `ldc_w` bytecode instruction loads a constant from the
/// current class' runtime constant pool onto the operand stack:
///
/// ```ignore
/// ldc index   // loads constant[index] onto the stack
/// ldc_w index // wide version, uses 2-byte index
/// ```
///
/// See:
/// - JVMS §5.1 (Runtime Constant Pool): https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-5.html
/// - JVMS §5.4.3 (Resolution): https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-5.html#jvms-5.4.3
/// - OpenJDK ConstantPool.cpp: https://github.com/openjdk/jdk/blob/master/src/hotspot/share/oops/constantPool.cpp
#[derive(Debug, Default)]
pub(in crate::vm) struct Ldc {
    cache: RwLock<HashMap<String, HashMap<ConstantPoolIndex, Value>>>,
}

type Value = Vec<i32>;
type ConstantPoolIndex = u16;

impl Ldc {
    pub fn resolve(&self, current_class: &str, index: ConstantPoolIndex) -> Result<i32, VmError> {
        if let Some(Some(value)) = self.cache.read().get(current_class).map(|m| m.get(&index)) {
            return Ok(value[0]);
        };

        // let class = CLASSES.get(current_class)?;
        todo!()
    }
}
