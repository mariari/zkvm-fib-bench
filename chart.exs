#!/usr/bin/env elixir
# Draw the three figures in RESULTS.md from the numbers in its tables.
#
#     elixir chart.exs     # writes prove_fib10000.svg, prove_bounds.svg, prove_default.svg
#
# Each figure has two panels on log axes, prover time and prover memory, one bar per
# (system, mode). The numbers are the same-rig medians recorded in RESULTS.md; edit the
# tables there, mirror the change here, rerun.
defmodule Chart do
  @system [{"zkFOL", "#2a78d6"}, {"RISC Zero", "#eb6834"}, {"SP1", "#1baf7a"}]
  @ink "#0b0b0b"
  @ink2 "#52514e"
  @muted "#898781"
  @grid "#e1e0d9"
  @axis "#c3c2b7"
  @surface "#fcfcfb"

  @w 960
  @panel_w 330
  @left 190
  @top 92
  @row 34
  @bar 20

  # {system, mode label, prove ms, prover memory MB, memory label}. zkFOL's prover memory
  # is its increment over the idle Erlang VM, under the ~10 MB noise floor; it is plotted
  # at the floor and labelled as a bound.
  @fib [
    {"zkFOL", "fibm, doubled", 4.82, 10, "&lt; 10 MB"},
    {"RISC Zero", "composite + fastdbl", 3690, 312},
    {"RISC Zero", "succinct + fastdbl", 14730, 1390},
    {"SP1", "core + fastdbl", 13320, 9390},
    {"SP1", "compressed + fastdbl", 49240, 17000}
  ]
  @default [
    {"zkFOL", "regsm", 1060, 10, "&lt; 10 MB"},
    {"zkFOL", "fib, doubled", 8.36, 10, "&lt; 10 MB"},
    {"RISC Zero", "succinct", 40670, 2300},
    {"SP1", "compressed", 50390, 17070}
  ]
  @bounds [
    {"zkFOL", "bounded", 1.53, 3, "~3 MB"},
    {"RISC Zero", "composite + bounds", 3700, 311},
    {"RISC Zero", "succinct + bounds", 14680, 1390},
    {"SP1", "core + bounds", 13230, 9360},
    {"SP1", "compressed + bounds", 49200, 17010}
  ]

  def main do
    figure(
      @fib,
      "fib(10,000) mod 7919, the same algorithm on every system",
      "fast doubling, 14 rounds instead of 10,000 steps; CPU only, AMD Ryzen 7 5700X; medians of 3",
      "prove_fib10000.svg"
    )

    figure(
      @bounds,
      "bounds check 10 ≤ x ≤ 100",
      "nothing to compute, so this is the fixed cost of a proof; CPU only, AMD Ryzen 7 5700X; medians of 3",
      "prove_bounds.svg"
    )

    figure(
      @default,
      "fib(10,000), each system's default route",
      "the published linear loop on the zkVMs; regsm like for like and plain fib on zkFOL; CPU only, AMD Ryzen 7 5700X; medians of 3",
      "prove_default.svg"
    )
  end

  defp figure(rows, title, subtitle, path) do
    h = @top + @row * length(rows) + 48

    legend =
      @system
      |> Enum.reverse()
      |> Enum.reduce({@w - 16, []}, fn {name, color}, {lx, acc} ->
        lx = lx - 8 - 7 * String.length(name)

        {lx - 24,
         [
           ~s(<rect x="#{lx}" y="14" width="10" height="10" rx="2" fill="#{color}"/>),
           ~s(<text x="#{lx + 16}" y="24" font-size="12" fill="#{@ink2}">#{name}</text>) | acc
         ]}
      end)
      |> elem(1)
      |> Enum.reverse()

    labels =
      Enum.with_index(rows, fn row, i ->
        y = @top + @row * i + @row / 2 + 4

        ~s(<text x="#{@left - 12}" y="#{y}" font-size="12" text-anchor="end" fill="#{@ink}">#{elem(row, 0)} #{elem(row, 1)}</text>)
      end)

    body =
      [
        ~s(<svg xmlns="http://www.w3.org/2000/svg" width="#{@w}" height="#{h}" viewBox="0 0 #{@w} #{h}" font-family="system-ui, -apple-system, 'Segoe UI', sans-serif">),
        ~s(<rect width="#{@w}" height="#{h}" fill="#{@surface}"/>),
        ~s(<text x="16" y="24" font-size="15" font-weight="600" fill="#{@ink}">#{title}</text>),
        ~s(<text x="16" y="42" font-size="12" fill="#{@ink2}">#{subtitle}</text>)
      ] ++
        legend ++
        labels ++
        panel(@left, rows, 2, 1, 100_000, [1, 10, 100, 1000, 10_000, 100_000], &fmt_ms/1, "prover time") ++
        panel(@left + @panel_w + 90, rows, 3, 1, 100_000, [1, 10, 100, 1000, 10_000, 100_000], &fmt_mb/1, "prover memory") ++
        ["</svg>"]

    File.write!(path, Enum.join(body, "\n") <> "\n")
  end

  defp panel(x0, rows, col, lo, hi, ticks, fmt, title) do
    span = :math.log10(hi) - :math.log10(lo)
    xs = fn v -> x0 + @panel_w * (:math.log10(v) - :math.log10(lo)) / span end
    y_top = @top
    y_bot = @top + @row * length(rows)

    tick_marks =
      Enum.flat_map(ticks, fn t ->
        x = f1(xs.(t))

        [
          ~s(<line x1="#{x}" y1="#{y_top - 6}" x2="#{x}" y2="#{y_bot}" stroke="#{@grid}" stroke-width="1"/>),
          ~s(<text x="#{x}" y="#{y_bot + 16}" font-size="11" text-anchor="middle" fill="#{@muted}">#{fmt.(t)}</text>)
        ]
      end)

    bars =
      rows
      |> Enum.with_index()
      |> Enum.flat_map(fn {row, i} ->
        v = elem(row, col)
        {_name, color} = List.keyfind(@system, elem(row, 0), 0)
        y = @top + @row * i + (@row - @bar) / 2
        x1 = xs.(v)
        w = x1 - x0
        label = if col == 3 and tuple_size(row) > 4, do: elem(row, 4), else: fmt.(v)

        [
          ~s(<path d="M#{x0},#{y} h#{f1(w - 4)} a4,4 0 0 1 4,4 v#{@bar - 8} a4,4 0 0 1 -4,4 h#{f1(-(w - 4))} z" fill="#{color}"/>),
          ~s(<text x="#{f1(x1 + 6)}" y="#{y + @bar / 2 + 4}" font-size="12" fill="#{@ink}">#{label}</text>)
        ]
      end)

    [~s(<text x="#{x0}" y="#{@top - 30}" font-size="13" font-weight="600" fill="#{@ink}">#{title}</text>)] ++
      tick_marks ++
      [~s(<line x1="#{x0}" y1="#{y_top - 6}" x2="#{x0}" y2="#{y_bot}" stroke="#{@axis}" stroke-width="1"/>)] ++
      bars ++
      [~s(<text x="#{x0 + @panel_w}" y="#{y_bot + 32}" font-size="11" text-anchor="end" fill="#{@muted}">log scale</text>)]
  end

  defp fmt_ms(v) when v >= 1000, do: "#{g3(v / 1000)} s"
  defp fmt_ms(v), do: "#{g3(v)} ms"

  defp fmt_mb(v) when v >= 1000, do: "#{g3(v / 1000)} GB"
  defp fmt_mb(v), do: "#{round(v)} MB"

  # three significant digits, trailing zeros dropped, as python's %.3g prints them
  defp g3(v) do
    digits = max(0, 2 - trunc(:math.floor(:math.log10(v))))
    r = Float.round(v / 1, digits)
    if r == trunc(r), do: "#{trunc(r)}", else: "#{r}"
  end

  defp f1(v), do: :erlang.float_to_binary(v / 1, decimals: 1)
end

Chart.main()
