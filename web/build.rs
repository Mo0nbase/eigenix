fn main() {
    // Define the API host and port at build time
    println!("cargo:rustc-env=API_HOST=nixlab");
    println!("cargo:rustc-env=API_PORT=3000");
}
