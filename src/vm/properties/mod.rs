use crate::{
    Args,
    vm::{Result, runtime::RuntimeError},
};
use indexmap::IndexMap;
use once_cell::sync::OnceCell;
use std::sync::LazyLock;

mod classpath;
mod os;

/// User-overridable system properties, resolved once at startup
///
/// Holds [DEFAULT_SYSTEM_PROPERTIES] extended with any matching entries from
/// [Args::system_properties]
static OVERRIDDEN_SYSTEM_PROPERTIES: OnceCell<IndexMap<&str, &str>> = OnceCell::new();

/// User-overridable platform properties, resolved once at startup
///
/// Holds [DEFAULT_PLATFORM_PROPERTIES] extended with any matching entries from [Args::system_properties]
static OVERRIDDEN_PLATFORM_PROPERTIES: OnceCell<IndexMap<&str, &str>> = OnceCell::new();

/// Default system properties returned by `java.lang.System.getProperties()`
///
/// These are the standard JVM system properties defined by the Java SE specification
/// Includes properties like:
/// - `java.version`, `java.home`, `java.class.path`
/// - `os.name`, `os.version`, `os.arch`
/// - `user.name`, `user.home`, `user.dir`
/// - `file.separator`, `path.separator`, `line.separator`
///
/// Note: Excludes platform-specific properties like `sun.*`, `display.*`, `format.*`
/// which are handled separately in `DEFAULT_PLATFORM_PROPERTIES`
///
/// See: https://docs.oracle.com/en/java/javase/17/docs/api/system-properties.html
static DEFAULT_SYSTEM_PROPERTIES: LazyLock<IndexMap<&str, &str>> = LazyLock::new(|| {
    IndexMap::from([
        ("java.version", "17"),
        ("java.vendor", "Ignis"),
        ("java.home", "."),
        ("java.class.version", "61.0"),
        ("java.vm.name", "Ignis"),
        ("java.vm.version", "0.1.0"),
        ("java.vm.vendor", "Ignis JVM"),
        ("java.vm.spec.name", "Java Virtual Machine Specification"),
        ("java.vm.spec.version", "17"),
        ("java.runtime.name", "Ignis JVM Runtime"),
        ("java.runtime.version", "17"),
        ("java.spec.version", "17"),
        ("java.spec.vendor", "Ignis JVM"),
        ("java.class.path", "."),
        ("file.separator", os::file_separator()),
        ("path.separator", os::path_separator()),
        ("line.separator", os::line_separator()),
        ("java.io.tmpdir", os::temp_dir()),
        ("java.library.path", ""),
        ("java.ext.dirs", ""),
    ])
});

/// Default platform properties for the JVM runtime
///
/// These are platform-specific properties that vary by OS and environment,
/// not part of the standard Java SE specification. Includes properties like:
/// - `sun.*` (vendor-specific JVM properties)
/// - `display.*` (locale display properties)
/// - `format.*` (locale format properties)
/// - `stderr.encoding`, `stdin.encoding`, `stdout.encoding` (console encodings)
/// - `native.encoding`, `sun.jnu.encoding` (native encoding)
///
/// See: OpenJDK source fragmentation:
/// - src/java.base/unix/native/libjava/java_props_md.c (Unix)
/// - src/java.base/windows/native/libjava/java_props_md.c (Windows)
/// - jdk/src/share/lib/net.properties (networking defaults)
///
/// Note: Excludes standard system properties which are handled separately in
/// `DEFAULT_SYSTEM_PROPERTIES`.
static DEFAULT_PLATFORM_PROPERTIES: LazyLock<IndexMap<&str, &str>> = LazyLock::new(|| {
    IndexMap::from([
        ("sun.cpu.endian", os::endianess()),
        ("sun.jnu.encoding", "UTF-8"),
        ("stderr.encoding", "cp437"),
        ("display.language", "en"),
    ])
});

impl Args<'static> {
    pub(crate) fn initialise_properties(&self) -> Result<()> {
        let mut overridden_system_props = DEFAULT_SYSTEM_PROPERTIES.clone();
        let mut overridden_platform_props = DEFAULT_PLATFORM_PROPERTIES.clone();

        for (&key, &value) in &self.system_properties {
            match overridden_platform_props.contains_key(&key) {
                true => overridden_platform_props.insert(key, value),
                false => overridden_system_props.insert(key, value),
            };
        }

        OVERRIDDEN_PLATFORM_PROPERTIES
            .set(overridden_platform_props)
            .map_err(|existing| {
                panic!("OVERRIDDEN_PLATFORM_PROPERTIES already initialised: {existing:?}")
            });
        OVERRIDDEN_SYSTEM_PROPERTIES
            .set(overridden_system_props)
            .map_err(|existing| {
                panic!("OVERRIDDEN_SYSTEM_PROPERTIES already initialised: {existing:?}")
            });

        Ok(())
    }
}
