use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/package.json");
    println!("cargo:rerun-if-changed=ui/vite.config.ts");
    println!("cargo:rerun-if-changed=ui/svelte.config.js");

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
