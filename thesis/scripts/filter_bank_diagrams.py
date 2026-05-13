import os
import re

import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np
from styles_config import (
    DPI,
    LINE_WIDTH,
    OUTPUT_DIR,
    apply_style,
)

apply_style()

FIG_WIDTH = 8
FIG_HEIGHT = 5

FS = 48000
Q15_MAX = 32767
Q15_MIN = -32768


def parse_rs_filter_file(path: str) -> tuple[int, list[dict]]:
    with open(path, "r") as f:
        text = f.read()

    ps_match = re.search(
        r"pub const OCTAVE_FILTER_POST_SHIFT\s*:\s*i8\s*=\s*(-?\d+)\s*;", text
    )
    if not ps_match:
        raise ValueError("OCTAVE_FILTER_POST_SHIFT not found in file.")
    post_shift = int(ps_match.group(1))

    coeffs_match = re.search(
        r"pub static OCTAVE_FILTER_COEFFS[^=]+=\s*\[(.*?)\]\s*;", text, re.DOTALL
    )
    if not coeffs_match:
        raise ValueError("OCTAVE_FILTER_COEFFS not found in file.")
    body = coeffs_match.group(1)

    # Split into individual band blocks: each starts with an optional comment
    # then a [ ... ] array literal.
    # Pattern: optional "// Band N (...)" comment followed by [ values ]
    band_pattern = re.compile(
        r"(?://\s*(Band\s+\d+[^\n]*)\n\s*)?\[([^\]]+)\]", re.DOTALL
    )

    bands = []
    for m in band_pattern.finditer(body):
        label_raw = m.group(1)
        values_raw = m.group(2)

        # Extract all Q15::from_bits(N) values in order
        bits = re.findall(r"Q15::from_bits\(\s*(-?\d+)\s*\)", values_raw)
        if len(bits) != 6:
            raise ValueError(
                f"Expected 6 coefficients per band, got {len(bits)}: {values_raw!r}"
            )
        coeffs = [int(b) for b in bits]

        label = label_raw.strip() if label_raw else f"Band {len(bands)}"
        label = (
            label.replace("Band", "Banda")
            .replace("Low-pass at", "Paso bajo,")
            .replace("Bandapass", "Paso banda,")
            .replace("High-pass at", "Paso alto,")
            .replace("-", " a ")
        )
        bands.append({"label": label, "coeffs": coeffs})

    if not bands:
        raise ValueError("No band arrays found in OCTAVE_FILTER_COEFFS.")

    return post_shift, bands


BAND_COLORS = [
    "#4E79A7",
    "#F28E2B",
    "#E15759",
    "#76B7B2",
    "#59A14F",
    "#B07AA1",
    "#EDC948",
    "#FF9DA7",
    "#9C755F",
    "#BAB0AC",
]
COMBINED_COLOR = "#333333"


def apply_biquad_q15(x_q15: np.ndarray, coeffs: list, post_shift: int) -> np.ndarray:
    b0, _pad, b1, b2, a1, a2 = [int(c) for c in coeffs]
    n = len(x_q15)
    y = np.zeros(n, dtype=np.int64)
    x1 = x2 = y1 = y2 = 0

    for i in range(n):
        xn = int(x_q15[i])
        acc = b0 * xn + b1 * x1 + b2 * x2 + a1 * y1 + a2 * y2
        yn = max(Q15_MIN, min(Q15_MAX, (acc >> 15) << post_shift))
        y[i] = yn
        x2, x1 = x1, xn
        y2, y1 = y1, yn

    return y.astype(np.int32)


def frequency_response_q15(
    coeffs: list,
    post_shift: int,
    n_freqs: int = 512,
    n_periods: int = 30,
    input_amplitude: int = 16384,
) -> tuple:
    freqs = np.logspace(np.log10(20), np.log10(FS / 2 - 1), n_freqs)
    magnitudes = np.zeros(n_freqs)

    for k, f in enumerate(freqs):
        samples_per_period = max(int(FS / f), 2)
        n_samples = n_periods * samples_per_period

        t = np.arange(n_samples) / FS
        x_q15 = np.clip(
            np.round(input_amplitude * np.sin(2 * np.pi * f * t)), Q15_MIN, Q15_MAX
        ).astype(np.int32)

        y_q15 = apply_biquad_q15(x_q15, coeffs, post_shift)

        y_ss = y_q15[n_samples // 2 :]
        output_amplitude = np.max(np.abs(y_ss))

        gain = output_amplitude / input_amplitude if output_amplitude > 0 else 1e-6
        magnitudes[k] = 20 * np.log10(max(gain, 1e-6))

    return freqs, magnitudes


def compute_combined_response(freqs: np.ndarray, all_magnitudes: list) -> np.ndarray:
    linear_sum = np.zeros(len(freqs))
    for mag_db in all_magnitudes:
        linear_sum += 10 ** (mag_db / 20)
    return 20 * np.log10(np.maximum(linear_sum, 1e-6))


def main():
    post_shift, bands = parse_rs_filter_file(
        "../sparklet/synth-engine/src/octave_filter/filter_coefficients.rs"
    )
    print(f"  post_shift = {post_shift}, bands = {len(bands)}")

    print("Computing Q15 frequency responses (swept-sine method)...")

    all_freqs = []
    all_magnitudes = []

    for band in bands:
        print(f"  {band['label']}...")
        freqs, mag = frequency_response_q15(band["coeffs"], post_shift)
        all_freqs.append(freqs)
        all_magnitudes.append(mag)

    combined_mag = compute_combined_response(all_freqs[0], all_magnitudes)

    fig, ax = plt.subplots(figsize=(FIG_WIDTH, FIG_HEIGHT))

    for i, band in enumerate(bands):
        color = BAND_COLORS[i % len(BAND_COLORS)]
        ax.semilogx(
            all_freqs[i],
            all_magnitudes[i],
            color=color,
            linewidth=LINE_WIDTH,
            label=band["label"],
            alpha=0.85,
        )

    ax.semilogx(
        all_freqs[0],
        combined_mag,
        color=COMBINED_COLOR,
        linewidth=LINE_WIDTH * 1.4,
        linestyle="--",
        label="Combinado (suma de las bandas)",
        alpha=1.0,
    )

    ax.set_xlim(20, FS / 2)
    ax.set_ylim(-60, 10)
    ax.set_xlabel("Frecuencia (Hz)")
    ax.set_ylabel("Ganancia (dB)")
    ax.set_title("Respuesta espectral del banco de filtros")

    ax.xaxis.set_major_formatter(
        ticker.FuncFormatter(lambda x, _: f"{int(x):,}".replace(",", "."))
    )
    ax.xaxis.set_major_locator(ticker.LogLocator(base=10, subs=[1, 2, 5], numticks=12))
    ax.xaxis.set_minor_locator(ticker.NullLocator())

    ax.legend(fontsize=8, loc="lower right")
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)

    plt.tight_layout(pad=1.5)

    os.makedirs(OUTPUT_DIR, exist_ok=True)
    out_path = os.path.join(OUTPUT_DIR, "octave_filter_response_q15.png")
    fig.savefig(
        out_path, dpi=DPI, bbox_inches="tight", facecolor="white", edgecolor="none"
    )
    print(f"Saved: {out_path}")
    plt.show()


if __name__ == "__main__":
    main()
