import os

import matplotlib.pyplot as plt
import numpy as np
from styles_config import DPI, LINE_COLOR, LINE_WIDTH, OUTPUT_DIR, PADDING, apply_style

apply_style()

# Figure dimensions (square plots side by side)
FIG_WIDTH = 8  # inches
FIG_HEIGHT = 8  # inches (square plots)


def db_to_amplitude(db):
    return 10 ** (db / 20)


def amplitude_to_db(amp):
    # Avoid log10(0) by clipping
    amp = np.maximum(amp, 1e-10)
    return 20 * np.log10(amp)


def create_figure_base():
    fig, axes = plt.subplots(2, 2, figsize=(FIG_WIDTH, FIG_HEIGHT))

    axes = axes.flatten()

    for ax in axes:
        ax.set_box_aspect(1)

    plt.tight_layout(pad=4)
    return fig, axes


def variation_1_linear():
    """
    Left: Linear amplitude (0 to 1)
    Right: Equivalent volume (-80 dB to 0 dB)
    """
    fig, axes = create_figure_base()

    t = np.linspace(0, 1, 500)

    # Left: Amplitude
    amplitude = t  # Linear 0 to 1
    axes[0].plot(t, amplitude, color=LINE_COLOR, linewidth=LINE_WIDTH)
    axes[0].set_xlim(-PADDING, 1 + PADDING)
    axes[0].set_ylim(-0.05, 1.05)
    axes[0].set_xlabel("Tiempo")
    axes[0].set_ylabel("Amplitud")
    axes[0].set_title("Ataque lineal en amplitud")

    # Right: Volume in dB
    volume = amplitude_to_db(amplitude)
    axes[1].plot(t, volume, color=LINE_COLOR, linewidth=LINE_WIDTH)
    axes[1].set_xlim(-PADDING, 1 + PADDING)
    axes[1].set_ylim(-45, 5)
    axes[1].set_xlabel("Tiempo")
    axes[1].set_ylabel("Volumen (dB)")
    axes[1].set_title("Ataque lineal en amplitud (dB)")

    # Left: Amplitude
    amplitude = 1 - t  # Linear 1 to 0
    axes[2].plot(t, amplitude, color=LINE_COLOR, linewidth=LINE_WIDTH)
    axes[2].set_xlim(-PADDING, 1 + PADDING)
    axes[2].set_ylim(-0.05, 1.05)
    axes[2].set_xlabel("Tiempo")
    axes[2].set_ylabel("Amplitud")
    axes[2].set_title("Decaimiento lineal en amplitud")

    # Right: Volume in dB
    volume = amplitude_to_db(amplitude)
    axes[3].plot(t, volume, color=LINE_COLOR, linewidth=LINE_WIDTH)
    axes[3].set_xlim(-PADDING, 1 + PADDING)
    axes[3].set_ylim(-45, 5)
    axes[3].set_xlabel("Tiempo")
    axes[3].set_ylabel("Volumen (dB)")
    axes[3].set_title("Decaimiento lineal en amplitud (dB)")

    return fig


def variation_2_linear_db():
    """
    Left: Linear volume (-80 dB to 0 dB)
    Right: Equivalent amplitude
    """
    fig, axes = create_figure_base()

    t = np.linspace(0, 1, 500)

    volume = -40 + 40 * t  # Linear from -40 to 0
    amplitude = db_to_amplitude(volume)

    # Left: Equivalent amplitude
    axes[0].plot(t, amplitude, color=LINE_COLOR, linewidth=LINE_WIDTH)
    axes[0].set_xlim(-PADDING, 1 + PADDING)
    axes[0].set_ylim(-0.05, 1.05)
    axes[0].set_xlabel("Tiempo")
    axes[0].set_ylabel("Amplitud")
    axes[0].set_title("Ataque lineal en volumen")

    # Right: Linear volume
    axes[1].plot(t, volume, color=LINE_COLOR, linewidth=LINE_WIDTH)
    axes[1].set_xlim(-PADDING, 1 + PADDING)
    axes[1].set_ylim(-45, 5)
    axes[1].set_xlabel("Tiempo")
    axes[1].set_ylabel("Volumen (dB)")
    axes[1].set_title("Ataque lineal en volumen (dB)")

    volume = -40 * t  # Linear from 0 to -40
    amplitude = db_to_amplitude(volume)

    # Left: Equivalent amplitude
    axes[2].plot(t, amplitude, color=LINE_COLOR, linewidth=LINE_WIDTH)
    axes[2].set_xlim(-PADDING, 1 + PADDING)
    axes[2].set_ylim(-0.05, 1.05)
    axes[2].set_xlabel("Tiempo")
    axes[2].set_ylabel("Amplitud")
    axes[2].set_title("Decaimiento lineal en volumen")

    # Right: Linear volume
    axes[3].plot(t, volume, color=LINE_COLOR, linewidth=LINE_WIDTH)
    axes[3].set_xlim(-PADDING, 1 + PADDING)
    axes[3].set_ylim(-45, 5)
    axes[3].set_xlabel("Tiempo")
    axes[3].set_ylabel("Volumen (dB)")
    axes[3].set_title("Decaimiento lineal en volumen (dB)")

    return fig


def variation_3_exponential():
    fig, axes = create_figure_base()

    t = np.linspace(0, 1, 500)
    r_t = 0.1
    tau = 1 / np.log((1 + r_t) / r_t)

    # Left: Amplitude with exponential formula
    T = 1 + r_t
    amplitude = T + (0 - T) * np.exp(-t / tau)

    axes[0].plot(t, amplitude, color=LINE_COLOR, linewidth=LINE_WIDTH)
    axes[0].set_xlim(-PADDING, 1 + PADDING)
    axes[0].set_ylim(-0.05, 1.05)
    axes[0].set_xlabel("Tiempo")
    axes[0].set_ylabel("Amplitud")
    axes[0].set_title("Ataque del modelo del condensador")

    # Right: Equivalent volume
    volume = amplitude_to_db(amplitude)
    axes[1].plot(t, volume, color=LINE_COLOR, linewidth=LINE_WIDTH)
    axes[1].set_xlim(-PADDING, 1 + PADDING)
    axes[1].set_ylim(-45, 5)
    axes[1].set_xlabel("Tiempo")
    axes[1].set_ylabel("Volumen (dB)")
    axes[1].set_title("Ataque del modelo del condensador (dB)")

    # Left: Amplitude with exponential formula
    T = 1 - (1 + r_t)
    amplitude = T + (1 - T) * np.exp(-t / tau)

    axes[2].plot(t, amplitude, color=LINE_COLOR, linewidth=LINE_WIDTH)
    axes[2].set_xlim(-PADDING, 1 + PADDING)
    axes[2].set_ylim(-0.05, 1.05)
    axes[2].set_xlabel("Tiempo")
    axes[2].set_ylabel("Amplitud")
    axes[2].set_title("Decaimiento del modelo del condensador")

    # Right: Equivalent volume
    volume = amplitude_to_db(amplitude)
    axes[3].plot(t, volume, color=LINE_COLOR, linewidth=LINE_WIDTH)
    axes[3].set_xlim(-PADDING, 1 + PADDING)
    axes[3].set_ylim(-45, 5)
    axes[3].set_xlabel("Tiempo")
    axes[3].set_ylabel("Volumen (dB)")
    axes[3].set_title("Decaimiento del modelo del condensador (dB)")

    return fig


def save_figure(fig, filename):
    """Save figure to output directory."""
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    filepath = os.path.join(OUTPUT_DIR, filename)
    fig.savefig(
        filepath, dpi=DPI, bbox_inches="tight", facecolor="white", edgecolor="none"
    )
    print(f"Saved: {filepath}")
    plt.close(fig)


if __name__ == "__main__":
    # Generate all three variations
    fig1 = variation_1_linear()
    save_figure(fig1, "adsr_amp_lineal.png")

    fig2 = variation_2_linear_db()
    save_figure(fig2, "adsr_vol_lineal.png")

    fig3 = variation_3_exponential()
    save_figure(fig3, "adsr_condensador.png")
