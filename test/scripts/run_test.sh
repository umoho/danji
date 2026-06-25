#!/usr/bin/env bash
# run_test.sh — danji-cli 一键测试脚本
# 用法: ./scripts/run_test.sh [run-id] [--gain-single N] [--gain-two-stage N] [--gain-chain N]
# 示例:
#   ./scripts/run_test.sh 2026-06-25_002
#   ./scripts/run_test.sh 2026-06-25_002 --gain-two-stage -20 --gain-chain -20

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TEST_DIR="$(dirname "$SCRIPT_DIR")"
DANJI="/Users/umoho/Devs/danji/target/release/danji-cli"
MODELS=("single" "two-stage" "chain")

# 默认增益
GAIN_SINGLE=0
GAIN_TWO_STAGE=0
GAIN_CHAIN=0

# 解析参数
RUN_ID=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --gain-single)
            GAIN_SINGLE="$2"
            shift 2
            ;;
        --gain-two-stage)
            GAIN_TWO_STAGE="$2"
            shift 2
            ;;
        --gain-chain)
            GAIN_CHAIN="$2"
            shift 2
            ;;
        --help|-h)
            echo "用法: ./scripts/run_test.sh [run-id] [--gain-single N] [--gain-two-stage N] [--gain-chain N]"
            echo "示例:"
            echo "  ./scripts/run_test.sh                              # 自动生成 run-id"
            echo "  ./scripts/run_test.sh 2026-06-25_002               # 指定 run-id"
            echo "  ./scripts/run_test.sh --gain-two-stage -20        # 自动生成 run-id，two-stage 使用 -20dB"
            echo "  ./scripts/run_test.sh 2026-06-25_002 --gain-chain -30  # 指定 run-id，chain 使用 -30dB"
            exit 0
            ;;
        *)
            RUN_ID="$1"
            shift
            ;;
    esac
done

# 生成 run-id: 传参则用参数，否则自动生成
if [ -z "$RUN_ID" ]; then
    DATE=$(date +%Y-%m-%d)
    EXISTING=$(ls -d "$TEST_DIR/analysis/reports/${DATE}_"* 2>/dev/null | wc -l | tr -d ' ')
    SEQ=$(printf "%03d" $((EXISTING + 1)))
    RUN_ID="${DATE}_${SEQ}"
fi

echo "============================================"
echo "  danji-cli 胆味测试"
echo "  Run ID: $RUN_ID"
echo "  增益: single=${GAIN_SINGLE}dB, two-stage=${GAIN_TWO_STAGE}dB, chain=${GAIN_CHAIN}dB"
echo "============================================"

# 检查 danji-cli
if [ ! -x "$DANJI" ]; then
    echo "错误: danji-cli 未找到，请先 cargo build --release"
    exit 1
fi

# 1. 生成测试音频
echo ""
echo "[1/4] 生成测试音频..."
uv run python "$SCRIPT_DIR/generate_test_audio.py"

# 2. 用 danji-cli 处理
echo ""
echo "[2/4] 处理测试音频..."
for model in "${MODELS[@]}"; do
    # 获取该模型的增益
    case "$model" in
        single)    gain=$GAIN_SINGLE ;;
        two-stage) gain=$GAIN_TWO_STAGE ;;
        chain)     gain=$GAIN_CHAIN ;;
    esac

    echo "  Model: $model (gain=${gain}dB)"
    for f in "$TEST_DIR"/input/sine_*.wav; do
        name=$(basename "$f")
        out="$TEST_DIR/output/$model/$name"
        $DANJI -i "$f" -o "$out" --model "$model" --gain="$gain" 2>&1 | sed 's/^/    /'
    done
done

# 3. 分析
echo ""
echo "[3/4] 分析结果..."
for model in "${MODELS[@]}"; do
    # 获取该模型的增益
    case "$model" in
        single)    gain=$GAIN_SINGLE ;;
        two-stage) gain=$GAIN_TWO_STAGE ;;
        chain)     gain=$GAIN_CHAIN ;;
    esac

    echo "  Model: $model (gain=${gain}dB)"
    uv run python "$SCRIPT_DIR/analyze.py" \
        --input-dir "$TEST_DIR/input" \
        --output-dir "$TEST_DIR/output/$model" \
        --run-id "$RUN_ID" \
        --model "$model" \
        --gain "$gain"
done

# 4. 生成图表
echo ""
echo "[4/4] 生成图表..."
uv run python "$SCRIPT_DIR/plot_results.py" \
    --models "${MODELS[@]}" \
    --run-id "$RUN_ID"

# 5. 编译 typst 报告
echo ""
echo "[5/5] 编译报告..."
REPORT_PDF="$TEST_DIR/reports/${RUN_ID}_report.pdf"
typst compile --root "$TEST_DIR" \
    "$TEST_DIR/template.typ" \
    "$REPORT_PDF" \
    --input "run-id=$RUN_ID" 2>&1 | sed 's/^/    /'
echo "  PDF: $REPORT_PDF"

echo ""
echo "============================================"
echo "  测试完成!"
echo "  数据: analysis/reports/$RUN_ID/"
echo "  图表: analysis/plots/$RUN_ID/"
echo "  报告: reports/${RUN_ID}_report.pdf"
echo "============================================"
