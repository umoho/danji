"""分析 danji-cli 处理音乐文件后的频谱特征变化"""

import argparse
import json
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
import soundfile as sf
from scipy.fft import rfft, rfftfreq


class NumpyEncoder(json.JSONEncoder):
    def default(self, obj):
        if isinstance(obj, (np.integer,)):
            return int(obj)
        if isinstance(obj, (np.floating,)):
            return float(obj)
        if isinstance(obj, np.ndarray):
            return obj.tolist()
        return super().default(obj)


def load_audio(path: Path) -> tuple[np.ndarray, int]:
    data, sr = sf.read(path, dtype="float32")
    if data.ndim > 1:
        data = data[:, 0]
    return data, sr


def compute_spectrum_db(signal: np.ndarray, sr: int) -> tuple[np.ndarray, np.ndarray]:
    N = len(signal)
    spectrum = np.abs(rfft(signal)) / N * 2
    freqs = rfftfreq(N, 1.0 / sr)
    mask = freqs > 0
    return freqs[mask], 20 * np.log10(spectrum[mask] + 1e-12)


def compute_band_energy(freqs: np.ndarray, db: np.ndarray,
                        low: float, high: float) -> float:
    mask = (freqs >= low) & (freqs < high)
    if not np.any(mask):
        return -100.0
    return float(np.mean(db[mask]))


def compute_spectral_centroid(freqs: np.ndarray, db: np.ndarray) -> float:
    power = 10.0 ** (db / 20.0)
    total = float(np.sum(power))
    if total == 0:
        return 0.0
    return float(float(np.sum(freqs * power)) / total)


def analyze_music(input_path: Path, output_path: Path) -> dict:
    in_sig, sr = load_audio(input_path)
    out_sig, sr_out = load_audio(output_path)

    assert sr == sr_out, f"采样率不匹配: {sr} vs {sr_out}"

    # 截取相同长度
    min_len = min(len(in_sig), len(out_sig))
    in_sig = in_sig[:min_len]
    out_sig = out_sig[:min_len]

    in_freqs, in_db = compute_spectrum_db(in_sig, sr)
    out_freqs, out_db = compute_spectrum_db(out_sig, sr)

    # 频段能量分析
    bands = [
        ("sub_bass", 20, 60),
        ("bass", 60, 250),
        ("low_mid", 250, 500),
        ("mid", 500, 2000),
        ("upper_mid", 2000, 4000),
        ("high", 4000, 8000),
        ("air", 8000, 20000),
    ]

    band_changes = {}
    for name, low, high in bands:
        in_energy = compute_band_energy(in_freqs, in_db, low, high)
        out_energy = compute_band_energy(out_freqs, out_db, low, high)
        band_changes[name] = round(out_energy - in_energy, 2)

    # 频谱质心变化
    in_centroid = compute_spectral_centroid(in_freqs, in_db)
    out_centroid = compute_spectral_centroid(out_freqs, out_db)

    # 谐波密度：高频能量占比
    in_high = compute_band_energy(in_freqs, in_db, 4000, 20000)
    out_high = compute_band_energy(out_freqs, out_db, 4000, 20000)
    in_total = compute_band_energy(in_freqs, in_db, 20, 20000)
    out_total = compute_band_energy(out_freqs, out_db, 20, 20000)

    # 削波检测：峰值电平
    in_peak_db = 20 * np.log10(np.max(np.abs(in_sig)) + 1e-12)
    out_peak_db = 20 * np.log10(np.max(np.abs(out_sig)) + 1e-12)

    # 动态范围
    in_rms = 20 * np.log10(np.sqrt(np.mean(in_sig ** 2)) + 1e-12)
    out_rms = 20 * np.log10(np.sqrt(np.mean(out_sig ** 2)) + 1e-12)

    return {
        "input": str(input_path),
        "output": str(output_path),
        "duration_sec": round(min_len / sr, 1),
        "sample_rate": sr,
        "band_changes_db": band_changes,
        "spectral_centroid": {
            "input_hz": round(in_centroid, 0),
            "output_hz": round(out_centroid, 0),
            "change_hz": round(out_centroid - in_centroid, 0),
        },
        "high_freq_ratio_db": round(out_high - in_high, 2),
        "peak_db": {
            "input": round(in_peak_db, 2),
            "output": round(out_peak_db, 2),
        },
        "rms_db": {
            "input": round(in_rms, 2),
            "output": round(out_rms, 2),
        },
    }


def plot_comparison(input_path: Path, output_path: Path, model: str,
                    title: str, output_dir: Path):
    in_sig, sr = load_audio(input_path)
    out_sig, _ = load_audio(output_path)

    min_len = min(len(in_sig), len(out_sig))
    in_sig = in_sig[:min_len]
    out_sig = out_sig[:min_len]

    in_freqs, in_db = compute_spectrum_db(in_sig, sr)
    out_freqs, out_db = compute_spectrum_db(out_sig, sr)

    fig, axes = plt.subplots(2, 1, figsize=(12, 8))

    # 频谱对比
    ax = axes[0]
    ax.semilogx(in_freqs, in_db, alpha=0.7, label="输入", linewidth=0.5)
    ax.semilogx(out_freqs, out_db, alpha=0.7, label=f"输出 ({model})", linewidth=0.5)
    ax.set_xlim(20, 20000)
    ax.set_xlabel("频率 (Hz)")
    ax.set_ylabel("幅度 (dB)")
    ax.set_title(f"{title} - 频谱对比")
    ax.legend()
    ax.grid(True, alpha=0.3)

    # 时域波形
    ax = axes[1]
    t = np.arange(min_len) / sr
    step = max(1, min_len // 10000)
    ax.plot(t[::step], in_sig[::step], alpha=0.7, label="输入", linewidth=0.5)
    ax.plot(t[::step], out_sig[::step], alpha=0.7, label=f"输出 ({model})", linewidth=0.5)
    ax.set_xlabel("时间 (s)")
    ax.set_ylabel("幅度")
    ax.set_title(f"{title} - 时域波形")
    ax.legend()
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    fig.savefig(output_dir / f"music_{model}_{title}.png", dpi=150)
    plt.close(fig)


def main():
    parser = argparse.ArgumentParser(description="分析音乐文件的胆味特征")
    parser.add_argument("--input-music", type=Path, required=True)
    parser.add_argument("--output-music", type=Path, required=True)
    parser.add_argument("--model", type=str, required=True)
    parser.add_argument("--title", type=str, default="")
    parser.add_argument("--output-dir", type=Path, default=None)
    args = parser.parse_args()

    if args.output_dir is None:
        args.output_dir = Path(__file__).resolve().parent.parent / "analysis" / "music"
    args.output_dir.mkdir(parents=True, exist_ok=True)

    print(f"分析 {args.title} ({args.model})...")
    print(f"  输入: {args.input_music}")
    print(f"  输出: {args.output_music}")

    result = analyze_music(args.input_music, args.output_music)

    # 保存 JSON
    json_path = args.output_dir / f"music_{args.model}_{args.title}.json"
    with open(json_path, "w") as f:
        json.dump(result, f, indent=2, ensure_ascii=False, cls=NumpyEncoder)
    print(f"  JSON: {json_path}")

    # 生成图表
    plot_comparison(args.input_music, args.output_music, args.model,
                    args.title, args.output_dir)
    print(f"  图表: {args.output_dir}/music_{args.model}_{args.title}.png")

    # 打印摘要
    print(f"\n{'='*60}")
    print(f"  {args.title} | {args.model}")
    print(f"{'='*60}")
    print(f"  时长: {result['duration_sec']}s")
    print(f"  频谱质心: {result['spectral_centroid']['input_hz']:.0f}Hz → "
          f"{result['spectral_centroid']['output_hz']:.0f}Hz "
          f"({result['spectral_centroid']['change_hz']:+.0f}Hz)")
    print(f"  高频能量变化: {result['high_freq_ratio_db']:+.2f} dB")
    print(f"  峰值电平: {result['peak_db']['input']:.1f}dB → {result['peak_db']['output']:.1f}dB")
    print(f"  RMS 电平: {result['rms_db']['input']:.1f}dB → {result['rms_db']['output']:.1f}dB")
    print(f"\n  频段能量变化:")
    for band, change in result["band_changes_db"].items():
        bar = "+" * max(0, int(change)) + "-" * max(0, int(-change))
        print(f"    {band:12s}: {change:+.2f} dB  {bar}")


if __name__ == "__main__":
    main()
