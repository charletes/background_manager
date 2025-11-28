fn main() {
    // Build the main Slint UI. This file includes all the other Slint files.
    slint_build::compile("ui/main.slint").expect("Slint build failed");
}
