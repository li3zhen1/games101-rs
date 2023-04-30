use std::env;
use std::process::Command;


const SHADER_NAME: &str = env!("SHADER_FILE_NAME");


fn main() {
    compile_shaders();
}

// xcrun -sdk macosx metal -gline-tables-only -frecord-sources -c shaders.metal -o shaders.air
// xcrun -sdk macosx metallib shaders.air -o shaders.metallib
fn compile_shaders() {

    println!("cargo:rerun-if-changed={SHADER_NAME}.metal");

    let output = Command::new("xcrun")
        .arg("-sdk")
        .arg("macosx")
        .arg("metal")
        .args(&["-gline-tables-only", "-frecord-sources"]) // add source for xcode debugger
        .args(&["-c", format!("{SHADER_NAME}.metal").as_str()])
        .args(&["-o", format!("{SHADER_NAME}.air").as_str()])
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    if !output.status.success() {
        panic!(
            r#"
stdout: {}
stderr: {}
"#,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        );
    }

    Command::new("xcrun")
        .arg("-sdk")
        .arg("macosx")
        .arg("metallib")
        .arg(format!("{SHADER_NAME}.air"))
        .args(&["-o", format!("{SHADER_NAME}.metallib").as_str()])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();


    

}
