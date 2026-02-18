use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/package.json");
    println!("cargo:rerun-if-changed=ui/vite.config.ts");
    println!("cargo:rerun-if-changed=ui/svelte.config.js");
    println!("cargo:rerun-if-env-changed=SKIP_UI_BUILD");

    // Skip UI build if SKIP_UI_BUILD environment variable is set
    // This is used during Yocto builds where the UI is built separately on the host
    if env::var("SKIP_UI_BUILD").is_ok() {
        println!("cargo:warning=Skipping UI build (SKIP_UI_BUILD is set)");
        return;
    }

    // Run pnpm build in the ui directory
    let status = Command::new("pnpm")
        .arg("build")
        .current_dir("ui")
        .status()
        .expect("Failed to execute pnpm build");

    if !status.success() {
        panic!("pnpm build failed");
    }
}
