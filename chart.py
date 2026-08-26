#!/usr/bin/env python3
"""Draw the two figures in RESULTS.md from the numbers in its tables.

    ./chart.py            # writes prove_fib10000.svg and prove_bounds.svg

Each figure has two panels on log axes, prover wall time and whole-process peak
RSS, one bar per (system, mode). The numbers are the same-rig medians recorded in
RESULTS.md; edit the tables there, mirror the change here, rerun.
"""
import math

SYSTEM = {"zkFOL": "#2a78d6", "RISC Zero": "#eb6834", "SP1": "#1baf7a"}
INK, INK2, MUTED, GRID, AXIS, SURFACE = (
    "#0b0b0b", "#52514e", "#898781", "#e1e0d9", "#c3c2b7", "#fcfcfb")

# (system, mode label, prove ms, peak RSS MB)
FIB = [
    ("zkFOL", "doubled mod", 4.82, 376),
    ("RISC Zero", "composite + fastdbl", 3690, 312),
    ("RISC Zero", "succinct + fastdbl", 14730, 1390),
    ("SP1", "core + fastdbl", 13320, 9390),
    ("SP1", "compressed + fastdbl", 49240, 17000),
]
BOUNDS = [
    ("zkFOL", "bounds", 1.53, 274),
    ("RISC Zero", "composite + bounds", 3700, 311),
    ("RISC Zero", "succinct + bounds", 14680, 1390),
    ("SP1", "core + bounds", 13230, 9360),
    ("SP1", "compressed + bounds", 49200, 17010),
]

W, PANEL_W, LEFT, TOP, ROW, BAR = 960, 330, 190, 92, 34, 20


def fmt_ms(v):
    return f"{v / 1000:.3g} s" if v >= 1000 else f"{v:.3g} ms"


def fmt_mb(v):
    return f"{v / 1000:.3g} GB" if v >= 1000 else f"{v:.0f} MB"


def panel(x0, rows, col, lo, hi, ticks, fmt, title):
    span = math.log10(hi) - math.log10(lo)
    xs = lambda v: x0 + PANEL_W * (math.log10(v) - math.log10(lo)) / span
    y_top, y_bot = TOP, TOP + ROW * len(rows)
    out = [f'<text x="{x0}" y="{TOP - 30}" font-size="13" font-weight="600" fill="{INK}">{title}</text>']
    for t in ticks:
        x = xs(t)
        out.append(f'<line x1="{x:.1f}" y1="{y_top - 6}" x2="{x:.1f}" y2="{y_bot}" stroke="{GRID}" stroke-width="1"/>')
        out.append(f'<text x="{x:.1f}" y="{y_bot + 16}" font-size="11" text-anchor="middle" fill="{MUTED}">{fmt(t)}</text>')
    out.append(f'<line x1="{x0}" y1="{y_top - 6}" x2="{x0}" y2="{y_bot}" stroke="{AXIS}" stroke-width="1"/>')
    for i, row in enumerate(rows):
        v, color = row[col], SYSTEM[row[0]]
        y, x1 = TOP + ROW * i + (ROW - BAR) / 2, xs(v)
        w = x1 - x0
        out.append(f'<path d="M{x0},{y} h{w - 4:.1f} a4,4 0 0 1 4,4 v{BAR - 8} a4,4 0 0 1 -4,4 h{-(w - 4):.1f} z" fill="{color}"/>')
        out.append(f'<text x="{x1 + 6:.1f}" y="{y + BAR / 2 + 4}" font-size="12" fill="{INK}">{fmt(v)}</text>')
    out.append(f'<text x="{x0 + PANEL_W}" y="{y_bot + 32}" font-size="11" text-anchor="end" fill="{MUTED}">log scale</text>')
    return out


def figure(rows, title, subtitle, path):
    h = TOP + ROW * len(rows) + 48
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{h}" viewBox="0 0 {W} {h}" '
           'font-family="system-ui, -apple-system, \'Segoe UI\', sans-serif">',
           f'<rect width="{W}" height="{h}" fill="{SURFACE}"/>',
           f'<text x="16" y="24" font-size="15" font-weight="600" fill="{INK}">{title}</text>',
           f'<text x="16" y="42" font-size="12" fill="{INK2}">{subtitle}</text>']
    lx = W - 16
    for name, color in reversed(SYSTEM.items()):
        lx -= 8 + 7 * len(name)
        out.append(f'<text x="{lx + 16}" y="24" font-size="12" fill="{INK2}">{name}</text>')
        out.append(f'<rect x="{lx}" y="14" width="10" height="10" rx="2" fill="{color}"/>')
        lx -= 24
    for i, row in enumerate(rows):
        y = TOP + ROW * i + ROW / 2 + 4
        out.append(f'<text x="{LEFT - 12}" y="{y}" font-size="12" text-anchor="end" fill="{INK}">{row[0]} {row[1]}</text>')
    out += panel(LEFT, rows, 2, 1, 100_000, [1, 10, 100, 1000, 10_000, 100_000], fmt_ms, "prover time")
    out += panel(LEFT + PANEL_W + 90, rows, 3, 100, 100_000, [100, 1000, 10_000, 100_000], fmt_mb, "peak RSS, whole process")
    out.append("</svg>")
    open(path, "w").write("\n".join(out) + "\n")


figure(FIB, "fib(10,000) mod 7919, fast doubling on every system",
       "same claim, same algorithm; CPU only, AMD Ryzen 7 5700X; medians of 3",
       "prove_fib10000.svg")
figure(BOUNDS, "bounds check 10 ≤ x ≤ 100, the smallest claim worth proving",
       "the fixed cost of a proof; CPU only, AMD Ryzen 7 5700X; medians of 3",
       "prove_bounds.svg")
