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

# The measuring calls, one per row (Examples.EBench):
#
#   measured_doubled_fibonacci_mod(10_000)     doubled mod
#   measured_bounds_check()                    bounds
#   measured_registers_fibonacci_mod(10_000)   registers mod
#   measured_registers_fibonacci(10_000)       registers
#   measured_doubled_fibonacci(10_000)         doubled
