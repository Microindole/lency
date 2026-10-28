use crate::lir_backend;
use anyhow::Result;
use std::fs;

/// 检查命令
pub fn cmd_check(input: &str) -> Result<()> {
    println!("Checking {} ...", input);

    if input.ends_with(".lir") {
        let source = fs::read_to_string(input)?;
        lir_backend::compile_lir_to_llvm_ir(&source)?;
        println!("No errors found");
        return Ok(());
    }

    let source = fs::read_to_string(input)?;
    match lency_driver::compile_file(input) {
        Ok(_) => {
            println!("No errors found");
            Ok(())
        }
        Err(e) => {
            e.emit(Some(input), Some(&source));
            std::process::exit(1);
        }
    }
}
