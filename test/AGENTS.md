# AGENTS.md — test 工作流程

## 运行测试

一键测试（推荐）：

```bash
./scripts/run_test.sh          # 自动生成 run-id (YYYY-MM-DD_NNN)
./scripts/run_test.sh 2026-06-25_001  # 指定 run-id
```

手动分步执行：

```bash
uv run python scripts/generate_test_audio.py  # 生成测试音频
# 用 danji-cli 处理（见 run_test.sh）
uv run python scripts/analyze.py --input-dir input --output-dir output/single --run-id 2026-06-25_001 --model single
uv run python scripts/plot_results.py --models single two-stage chain --run-id 2026-06-25_001
typst compile --root . template.typ reports/2026-06-25_001_report.pdf --input run-id=2026-06-25_001
```

## 新增测试信号

编辑 `scripts/generate_test_audio.py`，在 `tests` 列表中添加新频率：

```python
tests = [
    (100.0, "sine_100hz.wav"),
    (1000.0, "sine_1khz.wav"),
    (10000.0, "sine_10khz.wav"),
    # 添加新频率...
]
```

## 新增分析指标

编辑 `scripts/analyze.py`：
- 在 `analyze_pair()` 中计算新指标
- 在 `check_pass()` 中添加判定规则
- 输出会自动写入 JSON 报告

## 新增图表

编辑 `scripts/plot_results.py`：
- 添加新的绘图函数
- 在 `main()` 中调用

## typst 报告模板

`template.typ` 是数据驱动的报告模板：
- 自动从 `analysis/reports/<run-id>/` 读取 JSON 数据
- 自动遍历所有模型和频率
- 讨论和结论由 typst if-else 根据数据动态生成
- 编译时通过 `--input run-id=XXX` 指定数据批次

**中英文间距**：由 typst `cjk-latin-spacing:auto` 自动处理，不要手动加空格。变量名与中文之间用括号分隔：`#(model)模型`。

## 目录结构

```
test/
├── template.typ           # typst 报告模板
├── pyproject.toml         # Python 依赖
├── uv.lock                # 依赖锁定
├── scripts/
│   ├── run_test.sh        # 一键测试
│   ├── generate_test_audio.py
│   ├── analyze.py
│   └── plot_results.py
├── input/                 # 生成的测试音频 (.gitignore)
├── output/                # danji-cli 输出 (.gitignore)
├── analysis/
│   ├── reports/<run-id>/  # JSON 分析数据 (.gitignore)
│   └── plots/<run-id>/    # 生成的图表 (.gitignore)
└── reports/               # PDF 报告
```

## 合格标准

| 指标 | 合格标准 |
|------|----------|
| 二次谐波占比 | ≥ 60% |
| 谐波衰减 | 随阶数递减 |
| THD | 0.5% - 3% |
| 五次以上谐波 | < 5% |
