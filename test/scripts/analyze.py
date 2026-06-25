"""分析 danji-cli 输入/输出音频的谐波特征、THD、频谱对比"""

import argparse
import json
from pathlib import Path

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

ANALYSIS_DIR = Path(__file__).resolve().parent.parent / "analysis"
HARMONIC_ORDERS = 10


def load_audio(path: Path) -> tuple[np.ndarray, int]:
    data, sr = sf.read(path, dtype="float32")
    if data.ndim > 1:
        data = data[:, 0]
    return data, sr


def extract_harmonics(signal: np.ndarray, sr: int, f0: float) -> np.ndarray:
    N = len(signal)
    spectrum = np.abs(rfft(signal))
    freqs = rfftfreq(N, 1.0 / sr)

    amps = []
    for n in range(1, HARMONIC_ORDERS + 1):
        target = f0 * n
        idx = np.argmin(np.abs(freqs - target))
        amps.append(spectrum[idx])
    return np.array(amps)


def compute_thd(harmonics: np.ndarray) -> float:
    return float(np.sqrt(np.sum(harmonics[1:] ** 2)) / harmonics[0] * 100)


def compute_spectrum_db(signal: np.ndarray, sr: int) -> tuple[np.ndarray, np.ndarray]:
    N = len(signal)
    spectrum = np.abs(rfft(signal)) / N * 2
    freqs = rfftfreq(N, 1.0 / sr)
    mask = freqs > 0
    return freqs[mask], 20 * np.log10(spectrum[mask] + 1e-12)


def analyze_pair(input_path: Path, output_path: Path, f0: float) -> dict:
    in_sig, sr_in = load_audio(input_path)
    out_sig, sr_out = load_audio(output_path)

    in_harmonics = extract_harmonics(in_sig, sr_in, f0)
    out_harmonics = extract_harmonics(out_sig, sr_out, f0)

    in_thd = compute_thd(in_harmonics) if in_harmonics[0] > 0 else 0.0
    out_thd = compute_thd(out_harmonics)

    in_freqs, in_db = compute_spectrum_db(in_sig, sr_in)
    out_freqs, out_db = compute_spectrum_db(out_sig, sr_out)

    total_harmonic_power = np.sum(out_harmonics[1:] ** 2)
    second_ratio = float(out_harmonics[1] ** 2 / total_harmonic_power * 100) if total_harmonic_power > 0 else 0.0
    high_harmonic_ratio = float(np.sum(out_harmonics[4:] ** 2) / total_harmonic_power * 100) if total_harmonic_power > 0 else 0.0

    return {
        "input": str(input_path),
        "output": str(output_path),
        "f0_hz": f0,
        "sample_rate": sr_in,
        "input_thd_pct": round(in_thd, 4),
        "output_thd_pct": round(out_thd, 4),
        "input_harmonics_db": [round(20 * np.log10(a / in_harmonics[0] + 1e-12), 2) for a in in_harmonics],
        "output_harmonics_db": [round(20 * np.log10(a / out_harmonics[0] + 1e-12), 2) for a in out_harmonics],
        "second_harmonic_ratio_pct": round(second_ratio, 2),
        "high_harmonic_ratio_pct": round(high_harmonic_ratio, 2),
        "spectrum": {
            "input_freqs": [float(x) for x in in_freqs],
            "input_db": [float(x) for x in in_db],
            "output_freqs": [float(x) for x in out_freqs],
            "output_db": [float(x) for x in out_db],
        },
    }


def check_pass(result: dict) -> dict:
    thd = result["output_thd_pct"]
    sec_ratio = result["second_harmonic_ratio_pct"]
    high_ratio = result["high_harmonic_ratio_pct"]
    harmonics_db = result["output_harmonics_db"]

    衰减_ok = all(harmonics_db[i] >= harmonics_db[i + 1] for i in range(len(harmonics_db) - 2))

    checks = {
        "second_harmonic_dominant": sec_ratio >= 60,
        "thd_in_range": 0.5 <= thd <= 3.0,
        "high_harmonics_weak": high_ratio <= 5.0,
        "harmonics_decaying": 衰减_ok,
    }
    passed = sum(checks.values())
    if passed >= 4:
        verdict = "PASS"
    elif passed >= 3:
        verdict = "BORDERLINE"
    else:
        verdict = "FAIL"

    return {"checks": checks, "passed_count": passed, "verdict": verdict}


def main() -> None:
    parser = argparse.ArgumentParser(description="分析 danji-cli 胆味特征")
    parser.add_argument("--input-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--f0", type=float, default=1000.0, help="基频 (Hz)")
    parser.add_argument("--model", type=str, default="single")
    args = parser.parse_args()

    results = []
    for input_file in sorted(args.input_dir.glob("sine_*.wav")):
        output_file = args.output_dir / input_file.name
        if not output_file.exists():
            print(f"  跳过 {input_file.name}: 输出文件不存在")
            continue

        f0_str = input_file.stem.replace("sine_", "").replace("hz", "")
        if "k" in f0_str:
            f0 = float(f0_str.replace("k", "")) * 1000
        else:
            f0 = float(f0_str)

        print(f"  分析 {input_file.name} (f0={f0} Hz)...")
        result = analyze_pair(input_file, output_file, f0)
        verdict = check_pass(result)
        result["verdict"] = verdict
        results.append(result)

    report_dir = ANALYSIS_DIR / "reports"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_path = report_dir / f"analysis_{args.model}.json"
    with open(report_path, "w") as f:
        json.dump(results, f, indent=2, ensure_ascii=False, cls=NumpyEncoder)

    print(f"\n分析报告已保存: {report_path}")
    print(f"\n{'='*60}")
    print(f"  模型: {args.model}")
    print(f"{'='*60}")
    for r in results:
        v = r["verdict"]
        print(f"  {Path(r['input']).name}: THD={r['output_thd_pct']:.3f}%, "
              f"2nd={r['second_harmonic_ratio_pct']:.1f}%, "
              f"high={r['high_harmonic_ratio_pct']:.1f}% → {v['verdict']} ({v['passed_count']}/4)")


if __name__ == "__main__":
    main()
