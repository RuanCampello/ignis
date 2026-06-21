use indexmap::IndexMap;
use std::sync::LazyLock;
mod classpath;
mod os;

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
        ("java.vm.version", "1.0.0"),
        ("java.vm.vendor", "Ignis JVM"),
        ("java.vm.spec.name", "Java Virtual Machine Specification"),
        ("java.vm.spec.version", "17"),
        ("java.vm.spec.vendor", "Oracle Corporation"),
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

static DEFAULT_PLATFORM_PROPERTIES: LazyLock<IndexMap<&str, &str>> =
    LazyLock::new(|| IndexMap::default());
