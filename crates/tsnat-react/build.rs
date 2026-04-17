fn main() {
    // Ensure that rustc knows to look in /usr/local/lib where our setup script installed libSDL3.so
    println!("cargo:rustc-link-search=native=/usr/local/lib");
}
