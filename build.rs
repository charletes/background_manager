fn main() {
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
    
    // Also compile the gallery example if it's being built
    if std::path::Path::new("examples/slint_gallery/gallery.slint").exists() {
        slint_build::compile("examples/slint_gallery/gallery.slint").expect("Gallery Slint build failed");
    }
}
