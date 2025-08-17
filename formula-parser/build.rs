fn main() {
    // Get Ruby configuration
    let ruby_version = std::process::Command::new("ruby")
        .args(&["-e", "print RUBY_VERSION.gsub(/\\..+$/, '')"])
        .output()
        .expect("Failed to get Ruby version")
        .stdout;
    let ruby_version = String::from_utf8_lossy(&ruby_version);

    // Get Ruby library path
    let ruby_libdir = std::process::Command::new("ruby")
        .args(&["-e", "print RbConfig::CONFIG['libdir']"])
        .output()
        .expect("Failed to get Ruby libdir")
        .stdout;
    let ruby_libdir = String::from_utf8_lossy(&ruby_libdir);

    // Link against Ruby
    println!("cargo:rustc-link-search={}", ruby_libdir);
    println!("cargo:rustc-link-lib=ruby{}", ruby_version);
}