#!/usr/bin/env bash
# run_test.sh — danji-cli 一键测试脚本
# 用法: ./scripts/run_test.sh [run-id]
# 示例: ./scripts/run_test.sh 2026-06-25_001

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TEST_DIR="$(dirname "$SCRIPT_DIR")"
DANJI="/Users/umoho/Devs/danji/target/release/danji-cli"
MODELS=("single" "two-stage" "chain")

# 生成 run-id: 传参则用参数，否则自动生成
if [ $# -ge 1 ]; then
    RUN_ID="$1"
else
    DATE=$(date +%Y-%m-%d)
    # 查找当天已有最大序号
    EXISTING=$(ls -d "$TEST_DIR/analysis/reports/${DATE}_"* 2>/dev/null | wc -l | tr -d ' ')
    SEQ=$(printf "%03d" $((EXISTING + 1)))
    RUN_ID="${DATE}_${SEQ}"
fi

echo "============================================"
echo "  danji-cli 胆味测试"
echo "  Run ID: $RUN_ID"
echo "============================================"

# 检查 danji-cli
if [ ! -x "$DANJI" ]; then
    echo "错误: danji-cli 未找到，请先 cargo build --release"
    exit 1
fi

# 1. 生成测试音频
echo ""
echo "[1/3] 生成测试音频..."
uv run python "$SCRIPT_DIR/generate_test_audio.py"

# 2. 用 danji-cli 处理
echo ""
echo "[2/3] 处理测试音频..."
for model in "${MODELS[@]}"; do
    echo "  Model: $model"
    for f in "$TEST_DIR"/input/sine_*.wav; do
        name=$(basename "$f")
        out="$TEST_DIR/output/$model/$name"
        $DANJI -i "$f" -o "$out" --model "$model" 2>&1 | sed 's/^/    /'
    done
done

# 3. 分析
echo ""
echo "[3/3] 分析结果..."
for model in "${MODELS[@]}"; do
    echo "  Model: $model"
    uv run python "$SCRIPT_DIR/analyze.py" \
        --input-dir "$TEST_DIR/input" \
        --output-dir "$TEST_DIR/output/$model" \
        --run-id "$RUN_ID" \
        --model "$model"
done

# 4. 生成图表
echo ""
echo "[4/4] 生成图表..."
uv run python "$SCRIPT_DIR/plot_results.py" \
    --models "${MODELS[@]}" \
    --run-id "$RUN_ID"

echo ""
echo "============================================"
echo "  测试完成!"
echo "  报告: analysis/reports/$RUN_ID/"
echo "  图表: analysis/plots/$RUN_ID/"
echo "============================================"
