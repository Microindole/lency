#!/usr/bin/env python3
"""
检测代码中的 TODO 和 FIXME 标记

扫描项目中的源文件，查找未完成的任务标记。
"""

import os
import sys
from pathlib import Path
from typing import List, Tuple, Dict

# 配置
EXCLUDE_DIRS = {'.git', 'target', 'node_modules', '.gemini', 'assets', 'docs', 'scripts'}
EXTENSIONS = {'.rs', '.py', '.sh', '.md', '.brl'}
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

import re

def check_todos(root_dir: Path) -> Dict[str, List[Tuple[Path, int, str]]]:
    """检查 TODOs，返回 {tag: [(file, line_num, content)]}"""
    results = {tag: [] for tag in TAGS}
    
    # 构建正则匹配模式，确保匹配单词边界
    patterns = {tag: re.compile(rf'\b{tag}\b') for tag in TAGS}
    
    files = find_files(root_dir)
    
    for file_path in files:
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                for i, line in enumerate(f, 1):
                    for tag, pattern in patterns.items():
                        if pattern.search(line):
                            # 计算相对路径
                            rel_path = file_path.relative_to(root_dir)
                            results[tag].append((rel_path, i, line.strip()))
        except Exception:
            # 忽略各种编码错误等
            continue
            
    return results

def main():
    """主函数"""
    # 获取项目根目录
    script_dir = Path(__file__).parent
    project_root = script_dir.parent if script_dir.name == 'scripts' else script_dir
    
    print(f"🔍 扫描 TODO/FIXME 标记： {project_root}")
    print()
    
    results = check_todos(project_root)
    
    has_items = False
    total_count = 0
    
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
            
    if not has_items:
        print("✅ 没有发现未完成的标记！")
    else:
        print(f"📊 总计发现 {total_count} 个标记。")
        
    # 此脚本通常不应仅因为发现 TODO 就报错退出，除非是在严格的 CI 模式下
    # 这里我们只做报告，返回 0
    sys.exit(0)

if __name__ == '__main__':
    main()
