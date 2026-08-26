# The zkFOL definitions the benchmark ran, copied verbatim from zkfol 0.4.0
# (lib/examples/e_user.ex and e_doubling.ex). Each is a relation written with
# Zkfol.Lang; the pipeline turns it into a predicate, a witness, and a proof.

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

# fibm(x, v): the same recurrence reduced mod 7919 at every step. The
# "doubled mod" row is Doubling.rewrite(fibm, 10_000).
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
  regs(x - 1, a1, b1)
  a = a1 + b1
  b = a1
end

# regsm(x, a, b, q): the linear loop mod 7919, the zkbenchmarks.com program as
# a relation. q is the quotient witness of the reduction; the three guards pin
# the remainder to [0, 7919) and the quotient to N. The "registers mod" row.
defrel regsm(1, 1, 1, 0)

defrel regsm(x, a, b, q) do
  x > 1
  regsm(x - 1, a1, b1, q1)
  a1 + b1 = q * 7919 + a
  a < 7919
  a + 1 > 0
  q + 1 > 0
  b = a1
end

# bounded(x): 10 <= x <= 100 and nothing else. The "bounds" row.
defrel bounded(x) do
  x > 9
  x < 101
end

# The measuring calls, one per row (Examples.EBench):
#
#   measured_doubled_fibonacci_mod(10_000)     doubled mod
#   measured_bounds_check()                    bounds
#   measured_registers_fibonacci_mod(10_000)   registers mod
#   measured_registers_fibonacci(10_000)       registers
#   measured_doubled_fibonacci(10_000)         doubled
