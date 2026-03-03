#!/usr/bin/env python3
"""
检测过大的 Rust 文件

根据 Lency 项目规范，应避免单个文件过大，以保持代码可维护性。
此脚本扫描项目中的 Rust 文件，标记出超过指定行数的文件。
"""

import os
import sys
import argparse
from pathlib import Path
from typing import List, Tuple

# 配置
MAX_LINES_WARNING = 300   # 警告阈值
MAX_LINES_ERROR = 500     # 错误阈值
EXCLUDE_DIRS = {'.git', 'target', 'node_modules', '.gemini'}
EXTENSIONS = {'.rs', '.py', '.lcy'}

def in_scope(rel_path: Path, scope: str) -> bool:
    path = rel_path.as_posix()
    if scope == "all":
        return True
    if scope == "rust":
        return (
            path.startswith("crates/")
            or path.startswith("lib/")
            or path.startswith("tests/integration/")
        )
    if scope == "lency":
        return (
            path.startswith("lencyc/")
            or path.startswith("lib/")
            or path.startswith("tests/example/")
        )
    return False

def count_rust_code_lines(content: str) -> int:
    """计算 Rust/Lency 代码行数，排除注释（包括嵌套块注释）和空行"""
    # 替换块注释内容为空格，但保留换行符以维持行号/结构
    processed = []
    i = 0
    depth = 0
    while i < len(content):
        if content[i:i+2] == '/*':
            depth += 1
            processed.append('  ')
            i += 2
        elif content[i:i+2] == '*/':
            if depth > 0:
                depth -= 1
                processed.append('  ')
                i += 2
            else:
                # 孤立的 */，在 Rust 中可能是语法错误，这里直接保留
                processed.append('*/')
                i += 2
        else:
            if depth > 0:
                if content[i] == '\n':
                    processed.append('\n')
                else:
                    processed.append(' ')
            else:
                processed.append(content[i])
            i += 1
    
    clean_content = "".join(processed)
    code_lines = 0
    for line in clean_content.splitlines():
        # 处理单行注释
        # 这是一个近似实现，不考虑字符串内的 //
        if '//' in line:
            line = line.split('//')[0]
        
        if line.strip():
            code_lines += 1
    return code_lines

def count_python_code_lines(content: str) -> int:
    """计算 Python 代码行数，排除注释和空行"""
    code_lines = 0
    in_multiline = False
    quote_char = None
    
    for line in content.splitlines():
        stripped = line.strip()
        
        # 处理多行字符串（docstring）
        if in_multiline:
            if quote_char in stripped:
                in_multiline = False
            continue
        
        # 跳过空行
        if not stripped:
            continue
            
        # 跳过单行注释
        if stripped.startswith('#'):
            continue
            
        # 检测多行字符串开始（docstring）
        for quote in ('"""', "'''"):
            if quote in stripped:
                count = stripped.count(quote)
                if count == 1:
                    # 多行字符串开始
                    in_multiline = True
                    quote_char = quote
                    break
                # count >= 2 表示在同一行开始和结束，算一行代码
        
        code_lines += 1
    
    return code_lines

def count_lines(file_path: Path) -> int:
    """根据文件类型计算有效行数"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
            if file_path.suffix == '.rs' or file_path.suffix == '.lcy':
                return count_rust_code_lines(content)
            elif file_path.suffix == '.py':
                return count_python_code_lines(content)
            else:
                return sum(1 for line in content.splitlines() if line.strip())
    except Exception as e:
        print(f"⚠️  无法读取 {file_path}: {e}", file=sys.stderr)
        return 0

def find_code_files(root_dir: Path, scope: str = "all") -> List[Path]:
    """查找所有代码文件 (Rust, Python)"""
    code_files = []
    for root, dirs, files in os.walk(root_dir):
        # 过滤排除目录
        dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]
        
        for file in files:
            rel_root = Path(root).relative_to(root_dir)
            rel_path = rel_root / file
            if not in_scope(rel_path, scope):
                continue
            if any(file.endswith(ext) for ext in EXTENSIONS):
                code_files.append(Path(root) / file)
    
    return code_files

def check_file_sizes(root_dir: Path, scope: str = "all") -> Tuple[List, List]:
    """检查文件大小，返回 (warnings, errors)"""
    warnings = []
    errors = []
    
    code_files = find_code_files(root_dir, scope)
    
    for file_path in code_files:
        lines = count_lines(file_path)
        rel_path = file_path.relative_to(root_dir)
        
        if lines > MAX_LINES_ERROR:
            errors.append((rel_path, lines))
        elif  lines > MAX_LINES_WARNING:
            warnings.append((rel_path, lines))
    
    return warnings, errors

def main():
    """主函数"""
    parser = argparse.ArgumentParser(description="检查代码文件大小")
    parser.add_argument(
        "--scope",
        choices=["all", "rust", "lency"],
        default="all",
        help="检查范围: all/rust/lency (默认 all)",
    )
    args = parser.parse_args()

    # 获取项目根目录
    script_dir = Path(__file__).parent
    project_root = script_dir.parent if script_dir.name == 'scripts' else script_dir
    print(f"🔍 扫描代码文件 (Rust, Python)： {project_root} (scope={args.scope})")
    print(f"   警告阈值: {MAX_LINES_WARNING} 行")
    print(f"   错误阈值: {MAX_LINES_ERROR} 行")
    print()
    
    warnings, errors = check_file_sizes(project_root, args.scope)
    
    # 输出结果
    has_issues = False
    
    if errors:
        has_issues = True
        print("❌ 错误：以下文件过大 (需要重构):")
        for file_path, lines in sorted(errors, key=lambda x: x[1], reverse=True):
            print(f"   {file_path}: {lines} 行")
        print()
    
    if warnings:
        has_issues = True
        print("⚠️  警告：以下文件偏大 (建议考虑重构):")
        for file_path, lines in sorted(warnings, key=lambda x: x[1], reverse=True):
            print(f"   {file_path}: {lines} 行")
        print()
    
    if not has_issues:
        print("✅ 所有代码文件大小适中！")
    
    # 统计信息
    all_files = find_code_files(project_root, args.scope)
    total_lines = sum(count_lines(f) for f in all_files)
    avg_lines = total_lines // len(all_files) if all_files else 0
    
    print(f"📊 统计:")
    print(f"   总文件数: {len(all_files)}")
    print(f"   总行数: {total_lines}")
    print(f"   平均行数: {avg_lines}")
    print(f"   警告文件: {len(warnings)}")
    print(f"   错误文件: {len(errors)}")
    
    # 返回退出码
    sys.exit(1 if errors else 0)

if __name__ == '__main__':
    main()
