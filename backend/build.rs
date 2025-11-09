fn main() {
    // Define the API port at build time
    println!("cargo:rustc-env=API_PORT=3000");
}
