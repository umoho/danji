# Git 历史清理：debug 文件误提交修复

## 事故

`debug/solver_divergence_prompt.md` 在 `git add -A` 时被误加入两次：
- 第一次在 `eb3f93d`（line search commit）中
- 第二次在 `67dbd79`（TOL relax commit）中

两次都立即用 `git rm --cached` + `git commit` 尝试撤销，
在历史中留下了两个无意义的 `fix: remove debug prompt` commit。

## 修复

用 `git reset --soft <base>` + `git rebase -i` 回退到事故前的
base commit `759b401`（daemon socket），重新组织提交顺序：

1. `a526de3` feat: implement two-stage and chain amplifier models
2. `a0bb1c1` feat: add Newton line search with 10V step limit
3. `bc147eb` chore: devlog for Newton line search
4. `f444e46` perf: relax solver tolerance to 1e-6
5. `009d399` perf: increase MAX_ITER to 100

两个 `fix: remove` commit 从历史中完全消失，debug 文件从未出现在
git 记录中。用时约 15 分钟。

## 教训

- `git add -A` 会包含 `debug/` 等非源码目录，应改用 `git add src/` 或维护好 `.gitignore`
- 误提交后不要急着 `git commit` 修复，用 `git reset --soft HEAD~1` 撤回重做更干净
- 多人合作的分支不应 rebase 改写历史（但本地开发可以）
