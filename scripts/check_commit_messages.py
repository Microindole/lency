#!/usr/bin/env python3
"""Validate commit subjects in CI for push/pull_request events."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from typing import List

ALLOWED_TYPES = (
    "feat",
    "fix",
    "style",
    "refactor",
    "build",
    "ci",
    "docs",
    "perf",
    "test",
    "chore",
)

TYPE_PATTERN = "|".join(ALLOWED_TYPES)
# Conventional subject examples:
# - feat: add parser support
# - feat(vscode): improve completion
CONVENTIONAL_SUBJECT = re.compile(
    rf"^({TYPE_PATTERN})(\([a-z0-9][a-z0-9._/-]*\))?[：:]\s*\S(?:.*\S)?(?:\s+\(#\d+\))?$"
)

# GitHub merge subjects (default merge commit titles)
MERGE_SUBJECT = re.compile(
    r"^(Merge pull request #\d+ from .+|Merge branch '.+'(?: into .+)?|Merge remote-tracking branch '.+'(?: into .+)?|Merge tag '.+' of .+)$"
)


def run_git(*args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def load_event() -> dict:
    event_path = os.getenv("GITHUB_EVENT_PATH")
    if not event_path:
        print("ERROR: GITHUB_EVENT_PATH is not set.", file=sys.stderr)
        sys.exit(2)

    with open(event_path, "r", encoding="utf-8") as f:
        return json.load(f)


def list_commits_for_push(event: dict) -> List[str]:
    before = event.get("before", "")
    after = event.get("after", "")
    if not after:
        print("ERROR: push event payload missing 'after'.", file=sys.stderr)
        sys.exit(2)

    zero_sha = "0" * 40
    if before == zero_sha:
        return [after]

    try:
        output = run_git("rev-list", "--reverse", f"{before}..{after}")
        commits = [line for line in output.splitlines() if line]
        if commits:
            return commits
    except subprocess.CalledProcessError:
        # 在 force-push / 重写历史场景中，before..after 可能在本地不可达。
        # 降级到事件负载中的 commits 列表，避免校验流程直接崩溃。
        pass

    payload_commits = event.get("commits", [])
    commits = [c.get("id", "") for c in payload_commits if c.get("id")]
    if commits:
        return commits

    # 最后兜底：至少校验当前 after 提交本身。
    return [after]


def list_commits_for_pr(event: dict) -> List[str]:
    pr = event.get("pull_request", {})
    base_sha = pr.get("base", {}).get("sha", "")
    head_sha = pr.get("head", {}).get("sha", "")

    if not base_sha or not head_sha:
        print("ERROR: pull_request payload missing base/head SHA.", file=sys.stderr)
        sys.exit(2)

    output = run_git("rev-list", "--reverse", f"{base_sha}..{head_sha}")
    return [line for line in output.splitlines() if line]


def is_valid_subject(subject: str) -> bool:
    return bool(CONVENTIONAL_SUBJECT.match(subject) or MERGE_SUBJECT.match(subject))


def main() -> int:
    event_name = os.getenv("GITHUB_EVENT_NAME", "")
    if event_name == "push":
        event = load_event()
        commits = list_commits_for_push(event)
    elif event_name == "pull_request":
        event = load_event()
        commits = list_commits_for_pr(event)
    elif event_name == "workflow_dispatch":
        head_sha = run_git("rev-parse", "HEAD")
        commits = [head_sha]
    else:
        print(f"ERROR: unsupported event: {event_name}", file=sys.stderr)
        return 2

    if not commits:
        print("当前事件范围内没有需要校验的提交。")
        print("No commits to validate in this event range.")
        return 0

    print(f"开始校验提交主题，共 {len(commits)} 条。")
    print(f"Checking {len(commits)} commit(s) for subject format...")

    failures = []
    for idx, sha in enumerate(commits, start=1):
        subject = run_git("show", "-s", "--format=%s", sha)
        if not is_valid_subject(subject):
            failures.append((idx, sha, subject))

    if not failures:
        print("所有提交主题都符合规则。")
        print("All commit subjects are valid.")
        return 0

    print("\n检测到不合规提交主题：")
    print("Invalid commit subject(s) detected:")
    for idx, sha, subject in failures:
        print(f"- 第 {idx} 条 / Commit #{idx}: {sha[:12]}  {subject}")

    allowed = ", ".join(ALLOWED_TYPES)
    print("\n允许格式 / Allowed formats:")
    print(f"1) <type>: <subject>  ，其中 <type> ∈ [{allowed}]")
    print(f"   <type>: <subject>  where <type> in [{allowed}]")
    print("2) <type>(scope): <subject>  ，scope 仅允许 [a-z0-9._/-]")
    print("   <type>(scope): <subject>  with scope chars [a-z0-9._/-]")
    print("   允许尾缀 / Optional suffix: (#123)（用于 squash merge 生成标题）")
    print("   Optional suffix: (#123) (for squash-merge generated subjects)")
    print("3) 合并提交：Merge pull request / Merge branch / Merge remote-tracking branch / Merge tag ... of ...")
    print("   Merge subjects: Merge pull request / Merge branch / Merge remote-tracking branch / Merge tag ... of ...")
    print("\n示例 / Examples:")
    print("- feat: add parser diagnostics")
    print("- feat(vscode): improve completion cache")
    print("- feat(parser): support match expression (#123)")
    print("- Merge pull request #1 from Microindole/main")
    print("- Merge tag 'fsverity-for-linus' of git://git.kernel.org/pub/scm/fs/fsverity/linux")
    return 1


if __name__ == "__main__":
    sys.exit(main())
