fn main() {
    glib_build_tools::compile_resources(
        &["resources/gtk"],
        "resources/gtk/resources.gresource.xml",
        "waffle.gresource",
    );
}
