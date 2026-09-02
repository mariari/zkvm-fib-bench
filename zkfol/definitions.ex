# The relations the benchmark proved, as written for zkfol 0.4.0. Not runnable from this
# repo: they need the zkfol application (Zkfol.Lang). Read them here, run them from zkfol's iex.

defmodule Benchmarks do
  use Zkfol.Lang

  # fib(x, v): v is the x-th Fibonacci number, exact. The "doubled" row is this
  # relation after Zkfol.Doubling rewrote it to fast doubling.
  defrel fib(1, 1)
  defrel fib(2, 1)

  defrel fib(x, v) do
    x > 2
    fib(x - 1, v1)
    fib(x - 2, v2)
    v = v1 + v2
  end

  # fibm(x, v): the same recurrence reduced mod 7919 at every step.
  defrel fibm(1, 1)
  defrel fibm(2, 1)

  defrel fibm(x, v) do
    x > 2
    fibm(x - 1, v1)
    fibm(x - 2, v2)
    v = mod(v1 + v2, 7919)
  end

  # regs(x, a, b): the linear loop, exact. (a, b) are the two registers after
  # x steps. The "registers" row.
  defrel regs(1, 1, 1)

  defrel regs(x, a, b) do
    x > 1
    regs(x - 1, b, b1)
    a = b + b1
  end

  # regsm(x, a, b): the linear loop mod 7919, the zkbenchmarks.com program as a
  # relation. The "registers mod" row.
  defrel regsm(1, 1, 1)

  defrel regsm(x, a, b) do
    x > 1
    regsm(x - 1, b, b1)
    a = mod(b + b1, 7919)
  end

  # bounded(x): 10 <= x <= 100 and nothing else. The "bounds" row.
  defrel bounded(x) do
    x > 9
    x < 101
  end

  # The sudoku rows. column/2 peels a grid into its columns a head at a time via
  # heads/3, boxes/3 reads the b x b boxes off the grid's own cells, and sudoku/2
  # says every row, column and box is 1..n distinct. all_distinct lowers to
  # all_dif for AL and to Zkfol.Ast.distinct for the proof.
  defrel column([[] | _], [])

  defrel column(rows, [c | cs]) do
    heads(rows, c, rest)
    column(rest, cs)
  end

  defrel heads([], [], [])

  defrel heads([[h | t] | rs], [h | hs], [t | ts]) do
    heads(rs, hs, ts)
  end

  defrel boxes(n, rows, bs) do
    map(chunk(n), rows, runs)
    chunk(n, runs, bands)
    map(column, bands, stacks)
    map(map(concat), stacks, grouped)
    concat(grouped, bs)
  end

  defrel sudoku(x, blocks) do
    length(x, n)
    n = blocks ** 2
    blocks > 0
    each(each(between(1, n)), x)
    each(all_distinct, x)
    column(x, cols)
    each(all_distinct, cols)
    boxes(blocks, x, bs)
    each(all_distinct, bs)
  end

  # puzzle/2 is the seventeen-clue head the answer unifies against; solved/1 is
  # the measured row: the clues AND the rules, with nothing of the grid public.
  defrel puzzle(1, [[_, _, _, _, _, _, _, _, _],
                    [_, _, _, _, _, 3, _, 8, 5],
                    [_, _, 1, _, 2, _, _, _, _],
                    [_, _, _, 5, _, 7, _, _, _],
                    [_, _, 4, _, _, _, 1, _, _],
                    [_, 9, _, _, _, _, _, _, _],
                    [5, _, _, _, _, _, _, 7, 3],
                    [_, _, 2, _, 1, _, _, _, _],
                    [_, _, _, _, 4, _, _, _, 9]])

  defrel solved(x) do
    puzzle(1, x)
    sudoku(x, 3)
  end
end

# The measuring calls, one per row (Examples.EBench):
#
#   measured_doubled_fibonacci_mod(10_000)     doubled mod
#   measured_bounds_check()                    bounds
#   measured_registers_fibonacci_mod(10_000)   registers mod
#   measured_registers_fibonacci(10_000)       registers
#   measured_doubled_fibonacci(10_000)         doubled
#
# The sudoku row is not an EBench example; it is the front door directly
# (Examples.ESudoku supplies the relations and the grid):
#
#   Zkfol.eval!(Examples.ESudoku.solved(), Examples.ESudoku.act(), [])
#   |> Zkfol.Query.statement()
#   |> Zkfol.compile()
