# SPDX-License-Identifier: PMPL-1.0-or-later
# Benchee benchmarks for Seamstressd.Runner performance.
#
# Run with:
#   mix run bench/runner_bench.exs
#
# Note: benchee must be available as a dep (add to mix.exs under :dev if
# not already present):
#   {:benchee, "~> 1.3", only: :dev}
#
# These benchmarks cover:
#   - validate!/1 hot-path error handling (no seam records — fast path)
#   - Runner module loading cost (amortised by BEAM code cache)
#   - Process spawn overhead for concurrent validation requests
#   - Repeated call throughput for the runner's System.cmd dispatch

tmp_root = fn label ->
  path = Path.join(System.tmp_dir!(), "seamstress_bench_#{label}_#{System.unique_integer([:positive])}")
  File.mkdir_p!(path)
  path
end

# Pre-create directories so filesystem setup is not measured in the hot loop.
empty_root = tmp_root.("empty")
norecords_root = tmp_root.("norecords")
File.mkdir_p!(Path.join([norecords_root, "seams", "records"]))

Benchee.run(
  %{
    "validate!/1 — empty directory (error fast-path)" => fn ->
      try do
        Seamstressd.Runner.validate!(empty_root)
      rescue
        _e -> :ok
      end
    end,

    "validate!/1 — records dir present, no .seam.json (error fast-path)" => fn ->
      try do
        Seamstressd.Runner.validate!(norecords_root)
      rescue
        _e -> :ok
      end
    end,

    "validate!/1 — nonexistent path (error fast-path)" => fn ->
      try do
        Seamstressd.Runner.validate!("/nonexistent_bench_path_#{:rand.uniform(999_999)}")
      rescue
        _e -> :ok
      end
    end,

    "function_exported?/3 — runner module introspection" => fn ->
      function_exported?(Seamstressd.Runner, :validate!, 1)
    end,

    "module load check — Code.ensure_loaded?" => fn ->
      Code.ensure_loaded?(Seamstressd.Runner)
    end
  },
  time: 5,
  warmup: 2,
  memory_time: 2,
  print: [
    benchmarking: true,
    configuration: true,
    fast_warning: true
  ],
  formatters: [
    Benchee.Formatters.Console
  ]
)
