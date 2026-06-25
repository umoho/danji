"""生成 danji-cli 测试用正弦波音频"""

import numpy as np
import soundfile as sf
from pathlib import Path

INPUT_DIR = Path(__file__).resolve().parent.parent / "input"
SAMPLE_RATE = 44100
DURATION = 2.0  # 秒
AMPLITUDE = 0.5  # -6 dBFS


def generate_sine(freq: float, output_path: Path) -> None:
    t = np.linspace(0, DURATION, int(SAMPLE_RATE * DURATION), endpoint=False)
    signal = AMPLITUDE * np.sin(2 * np.pi * freq * t)
    sf.write(output_path, signal.astype(np.float32), SAMPLE_RATE)
    print(f"  {output_path.name}: {freq} Hz, {DURATION}s, {SAMPLE_RATE} Hz")


def main() -> None:
    INPUT_DIR.mkdir(parents=True, exist_ok=True)

    tests = [
        (100.0, "sine_100hz.wav"),
        (1000.0, "sine_1khz.wav"),
        (10000.0, "sine_10khz.wav"),
    ]

    print("生成测试音频:")
    for freq, filename in tests:
        generate_sine(freq, INPUT_DIR / filename)

    print(f"\n共生成 {len(tests)} 个文件到 {INPUT_DIR}")


if __name__ == "__main__":
    main()
