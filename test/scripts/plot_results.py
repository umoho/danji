"""生成 danji-cli 测试分析图表"""

import argparse
import json
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

ANALYSIS_DIR = Path(__file__).resolve().parent.parent / "analysis"
PLOTS_DIR = ANALYSIS_DIR / "plots"

plt.rcParams["font.family"] = ["Arial", "sans-serif"]
plt.rcParams["figure.dpi"] = 150
plt.rcParams["figure.figsize"] = (12, 8)


def plot_harmonic_bars(result: dict, model: str) -> None:
    harmonics_db = result["output_harmonics_db"]
    orders = list(range(1, len(harmonics_db) + 1))

    fig, ax = plt.subplots()
    colors = ["#2196F3"] + ["#FF9800"] * (len(orders) - 1)
    ax.bar(orders, harmonics_db, color=colors, edgecolor="white", linewidth=0.5)
    ax.set_xlabel("Harmonic Order")
    ax.set_ylabel("Amplitude (dB, relative to fundamental)")
    ax.set_title(f"Harmonic Spectrum - {model}\nf0={result['f0_hz']} Hz, THD={result['output_thd_pct']:.3f}%")
    ax.set_xticks(orders)
    ax.axhline(y=0, color="gray", linestyle="--", linewidth=0.5)
    ax.grid(axis="y", alpha=0.3)

    f0_name = Path(result["input"]).stem
    out_path = PLOTS_DIR / f"harmonics_{model}_{f0_name}.png"
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)
    print(f"  {out_path.name}")


def plot_spectrum_comparison(result: dict, model: str) -> None:
    spec = result["spectrum"]
    in_freqs = np.array(spec["input_freqs"])
    in_db = np.array(spec["input_db"])
    out_freqs = np.array(spec["output_freqs"])
    out_db = np.array(spec["output_db"])

    mask_in = in_freqs <= 20000
    mask_out = out_freqs <= 20000

    fig, ax = plt.subplots()
    ax.plot(in_freqs[mask_in], in_db[mask_in], label="Input", alpha=0.7, linewidth=0.8)
    ax.plot(out_freqs[mask_out], out_db[mask_out], label="Output", alpha=0.7, linewidth=0.8)
    ax.set_xlabel("Frequency (Hz)")
    ax.set_ylabel("Magnitude (dB)")
    ax.set_title(f"Spectrum Comparison - {model}\nf0={result['f0_hz']} Hz")
    ax.set_xscale("log")
    ax.set_xlim(20, 20000)
    ax.legend()
    ax.grid(alpha=0.3)

    f0_name = Path(result["input"]).stem
    out_path = PLOTS_DIR / f"spectrum_{model}_{f0_name}.png"
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)
    print(f"  {out_path.name}")


def plot_thd_comparison(all_results: dict[str, list], model: str) -> None:
    freqs = []
    thds = []
    for r in all_results[model]:
        freqs.append(r["f0_hz"])
        thds.append(r["output_thd_pct"])

    if not freqs:
        return

    fig, ax = plt.subplots()
    ax.bar(range(len(freqs)), thds, color="#E91E63", edgecolor="white")
    ax.set_xticks(range(len(freqs)))
    ax.set_xticklabels([f"{f:.0f} Hz" for f in freqs])
    ax.set_ylabel("THD (%)")
    ax.set_title(f"Total Harmonic Distortion - {model}")
    ax.axhline(y=0.5, color="green", linestyle="--", linewidth=0.8, label="Lower bound (0.5%)")
    ax.axhline(y=3.0, color="red", linestyle="--", linewidth=0.8, label="Upper bound (3.0%)")
    ax.legend()
    ax.grid(axis="y", alpha=0.3)

    out_path = PLOTS_DIR / f"thd_{model}.png"
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)
    print(f"  {out_path.name}")


def plot_verdict_summary(all_results: dict[str, list]) -> None:
    models = list(all_results.keys())
    verdicts = {"PASS": 0, "BORDERLINE": 0, "FAIL": 0}

    for model in models:
        for r in all_results[model]:
            v = r["verdict"]["verdict"]
            verdicts[v] = verdicts.get(v, 0) + 1

    fig, ax = plt.subplots()
    labels = list(verdicts.keys())
    sizes = [verdicts[k] for k in labels]
    colors_map = {"PASS": "#4CAF50", "BORDERLINE": "#FFC107", "FAIL": "#F44336"}
    colors = [colors_map[k] for k in labels]

    if sum(sizes) > 0:
        ax.pie(sizes, labels=labels, colors=colors, autopct="%1.0f%%", startangle=90)
    ax.set_title("danji-cli Test Verdict Summary")

    out_path = PLOTS_DIR / "verdict_summary.png"
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)
    print(f"  {out_path.name}")


def main() -> None:
    parser = argparse.ArgumentParser(description="生成分析图表")
    parser.add_argument("--models", nargs="+", default=["single", "two-stage", "chain"])
    args = parser.parse_args()

    PLOTS_DIR.mkdir(parents=True, exist_ok=True)

    all_results = {}
    for model in args.models:
        report_path = ANALYSIS_DIR / "reports" / f"analysis_{model}.json"
        if not report_path.exists():
            print(f"  跳过 {model}: 报告不存在")
            continue
        with open(report_path) as f:
            all_results[model] = json.load(f)

    for model, results in all_results.items():
        print(f"\n{model}:")
        for r in results:
            plot_harmonic_bars(r, model)
            plot_spectrum_comparison(r, model)
        plot_thd_comparison(all_results, model)

    if all_results:
        plot_verdict_summary(all_results)

    print(f"\n图表已保存到 {PLOTS_DIR}")


if __name__ == "__main__":
    main()
