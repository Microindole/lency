#!/usr/bin/env python3
"""
检测代码中的 TODO 和 FIXME 标记

扫描项目中的源文件，查找未完成的任务标记。
特别关注带有 @expect-error 的预期失败测试文件中的 TODO/FIXME。
"""

import os
import sys
import re
from pathlib import Path
from typing import List, Tuple, Dict

# 配置
EXCLUDE_DIRS = {'.git', 'target', 'node_modules', '.gemini', 'assets', 'docs', 'scripts'}
EXTENSIONS = {'.rs', '.py', '.sh', '.md', '.lcy'}
TAGS = {'TODO', 'FIXME', 'XXX'}

def find_files(root_dir: Path) -> List[Path]:
    """查找所有源代码文件"""
    found_files = []
    for root, dirs, files in os.walk(root_dir):
        # 过滤排除目录
        dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]
        
        for file in files:
            if any(file.endswith(ext) for ext in EXTENSIONS):
                found_files.append(Path(root) / file)
    
    return found_files

def check_todos(root_dir: Path) -> Tuple[Dict[str, List], List[Tuple]]:
    """
    检查 TODOs，返回两个结果：
    1. 普通 TODO/FIXME: {tag: [(file, line_num, content)]}
    2. 预期失败测试中的 TODO/FIXME: [(file, tag, reason)]
    """
    results = {tag: [] for tag in TAGS}
    expected_failures = []  # (file, tag, reason)
    
    # 构建正则匹配模式，确保匹配单词边界
    patterns = {tag: re.compile(rf'\b{tag}\b') for tag in TAGS}
    # 匹配 @expect-error 后面的内容
    expect_error_pattern = re.compile(r'@expect-error:\s*(TODO|FIXME)\s*-\s*(.+)')
    
    files = find_files(root_dir)
    
    for file_path in files:
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
                
                # 检查文件开头是否有 @expect-error: TODO/FIXME
                for i, line in enumerate(lines[:5]):  # 只检查前5行
                    match = expect_error_pattern.search(line)
                    if match:
                        tag = match.group(1)
                        reason = match.group(2).strip()
                        rel_path = file_path.relative_to(root_dir)
                        expected_failures.append((rel_path, tag, reason))
                        break  # 每个文件只记录一次
                
                # 检查普通 TODO/FIXME（跳过 @expect-error 行）
                for i, line in enumerate(lines, 1):
                    if '@expect-error' in line:
                        continue  # 跳过 @expect-error 行，它们会单独处理
                    for tag, pattern in patterns.items():
                        if pattern.search(line):
                            rel_path = file_path.relative_to(root_dir)
                            results[tag].append((rel_path, i, line.strip()))
        except Exception:
            # 忽略各种编码错误等
            continue
            
    return results, expected_failures

def main():
    """主函数"""
    # 获取项目根目录
    script_dir = Path(__file__).parent
    project_root = script_dir.parent if script_dir.name == 'scripts' else script_dir
    
    print(f"🔍 扫描 TODO/FIXME 标记： {project_root}")
    print()
    
    results, expected_failures = check_todos(project_root)
    
    has_items = False
    total_count = 0
    
    # 先显示普通 TODO/FIXME
    for tag in TAGS:
        items = results[tag]
        if items:
            has_items = True
            count = len(items)
            total_count += count
            
            icon = "🔴" if tag == "FIXME" else "📝"
            print(f"{icon} Found {count} {tag}s:")
            
            for file_path, line_num, content in items:
                # 截断过长内容
                if len(content) > 60:
                    content = content[:57] + "..."
                print(f"   {file_path}:{line_num:<4} {content}")
            print()
    
    # 显示预期失败测试中的 TODO/FIXME
    if expected_failures:
        has_items = True
        todos = [(f, t, r) for f, t, r in expected_failures if t == 'TODO']
        fixmes = [(f, t, r) for f, t, r in expected_failures if t == 'FIXME']
        
        if todos:
            print(f"📋 预期失败测试 (功能未实现 - {len(todos)} 个):")
            for file_path, tag, reason in todos:
                print(f"   {file_path}: {reason}")
            print()
            total_count += len(todos)
        
        if fixmes:
            print(f"🐛 预期失败测试 (需要修复 Bug - {len(fixmes)} 个):")
            for file_path, tag, reason in fixmes:
                print(f"   {file_path}: {reason}")
            print()
            total_count += len(fixmes)
            
    if not has_items:
        print("✅ 没有发现未完成的标记！")
    else:
        print(f"📊 总计发现 {total_count} 个标记。")
        
    # 此脚本通常不应仅因为发现 TODO 就报错退出，除非是在严格的 CI 模式下
    # 这里我们只做报告，返回 0
    sys.exit(0)

if __name__ == '__main__':
    main()
