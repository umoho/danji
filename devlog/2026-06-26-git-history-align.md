# Git 历史调整：对齐远程分支

## 背景

之前用 `git rebase -i` 改写了本地历史（删除误提交的 debug 文件、重排
commit 顺序），导致本地与 `origin/main` 分叉。`git push` 被拒绝，
因为远程的 `7fd0cfe feat: implement two-stage and chain amplifier models`
与本地 rebase 后的版本（`a526de3`）是不同 hash。

不能用 force push（用户要求保留远程基线）。

## 方案

1. `git reset --hard origin/main` 回到远程
2. `git cherry-pick` 本地 7 个 commit 到远程的 `7fd0cfe` 之上

```
git reset --hard origin/main
git cherry-pick a0bb1c1 bc147eb f444e46 009d399 45dfdd2 7d1b832 700fe03
```

## 结果

无冲突，7 个 commit 全部成功接上。历史：

```
7fd0cfe feat: implement two-stage...                    ← 远程原始
97bf265 feat: add Newton line search
07eec69 chore: devlog for Newton line search
97132ef perf: relax solver tolerance to 1e-6
380ffce perf: increase MAX_ITER to 100
3e0b884 chore: devlog for git history cleanup
a86d63e fix(danji-realtime): apply input gain + output atten
f3ca245 chore: fix clippy warnings; add missing bench calls
```

`git push` 现在可以正常 fast-forward。
