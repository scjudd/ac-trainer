fn main() {
    // Needed for winapi::GetKeyState
    println!("cargo:rustc-link-lib=dylib=user32");
}
