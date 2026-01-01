use anyhow::Result;
use beryl_driver::compile_file;
use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser)]
#[command(name = "berylc")]
#[command(about = "Beryl 编译器 - 简洁、规范、清晰", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 编译 Beryl 源文件为 LLVM IR
    Compile {
        /// 输入文件
        input: String,

        /// 输出文件 (默认: a.ll)
        #[arg(short, long, default_value = "a.ll")]
        output: String,
    },

    /// 编译并运行 Beryl 程序
    Run {
        /// 输入文件
        input: String,
    },

    /// 检查语法和语义错误
    Check {
        /// 输入文件
        input: String,
    },

    /// 编译并生成可执行文件
    Build {
        /// 输入文件
        input: String,

        /// 输出文件 (默认: a.out)
        #[arg(short, long, default_value = "a.out")]
        output: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { input, output } => cmd_compile(&input, &output)?,
        Commands::Run { input } => cmd_run(&input)?,
        Commands::Check { input } => cmd_check(&input)?,
        Commands::Build { input, output } => cmd_build(&input, &output)?,
    }

    Ok(())
}

/// 编译命令
fn cmd_compile(input: &str, output: &str) -> Result<()> {
    println!("📦 编译 {} ...", input);

    let result = compile_file(input)?;

    fs::write(output, result.ir)?;
    println!("✅ 成功生成 {}", output);

    Ok(())
}

/// 运行命令
fn cmd_run(input: &str) -> Result<()> {
    println!("🚀 运行 {} ...", input);

    // 1. 编译
    let result = compile_file(input)?;

    // 2. 写临时文件
    let temp_ir = "/tmp/beryl_temp.ll";
    fs::write(temp_ir, result.ir)?;

    // 3. 使用 lli 运行 LLVM IR
    let output = std::process::Command::new("lli").arg(temp_ir).output()?;

    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }

    Ok(())
}

/// 检查命令
fn cmd_check(input: &str) -> Result<()> {
    println!("🔍 检查 {} ...", input);

    match compile_file(input) {
        Ok(_) => {
            println!("✅ 无错误");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ 发现错误:\n{}", e);
            std::process::exit(1);
        }
    }
}

/// 构建命令 - 生成可执行文件
fn cmd_build(input: &str, output: &str) -> Result<()> {
    println!("🔨 构建 {} ...", input);

    // 1. 编译为 LLVM IR
    let result = compile_file(input)?;
    let temp_ll = "/tmp/beryl_temp.ll";
    fs::write(temp_ll, result.ir)?;

    // 2. 使用 llc 生成目标文件
    println!("  ⚙️  生成目标文件...");
    let temp_obj = "/tmp/beryl_temp.o";
    let llc_status = std::process::Command::new("llc-15")
        .args(["-filetype=obj", temp_ll, "-o", temp_obj])
        .status()?;

    if !llc_status.success() {
        anyhow::bail!("llc 编译失败");
    }

    // 3. 使用 gcc 链接
    println!("  🔗 链接可执行文件...");
    let gcc_status = std::process::Command::new("gcc")
        .args([temp_obj, "-o", output])
        .status()?;

    if !gcc_status.success() {
        anyhow::bail!("链接失败");
    }

    println!("✅ 成功生成可执行文件: {}", output);
    Ok(())
}
