use std::process::Command;

fn main() {
    // Step 1: Run the preprocessor
    let status = Command::new("cargo")
        .args(["run", "-p", "compile"])
        .status()
        .expect("Failed to run compile");

    if !status.success() {
        panic!("Compile step failed");
    }

    // Step 2: Run the runtime
    let status = Command::new("cargo")
        .args(["run", "-p", "runtime"])
        .status()
        .expect("Failed to run runtime");

    if !status.success() {
        panic!("Runtime step failed");
    }
}
