# SPDX-License-Identifier: PMPL-1.0-or-later
# Property-based tests for Seamstressd.Runner.
#
# These tests use manual generation loops (no StreamData dependency required)
# to verify that runner invariants hold across a wide range of inputs:
#
#   1. Non-existent paths always raise RuntimeError.
#   2. Paths that exist but contain no seam records always raise RuntimeError.
#   3. Error messages are always non-empty strings.
#   4. The runner never crashes the process (raises, not exits).
#   5. The behaviour is deterministic for a given input.

defmodule Seamstressd.PropertyTest do
  use ExUnit.Case, async: true

  # ---------------------------------------------------------------------------
  # Generators
  # ---------------------------------------------------------------------------

  # Generate a stream of random-looking (but invalid) filesystem paths.
  # These are guaranteed not to exist so the runner must always raise.
  defp random_nonexistent_paths(count) do
    for i <- 1..count do
      "/tmp/seamstress_prop_#{i}_#{System.unique_integer([:positive])}_#{:rand.uniform(999_999)}"
    end
  end

  # Generate a stream of empty temporary directories (exist, but no records).
  defp empty_temp_dirs(count) do
    for i <- 1..count do
      path = Path.join(System.tmp_dir!(), "seamstress_prop_empty_#{i}_#{System.unique_integer([:positive])}")
      File.mkdir_p!(path)
      path
    end
  end

  # Generate paths with seams/records/ present but no .seam.json files.
  defp dirs_with_records_dir_but_no_files(count) do
    for i <- 1..count do
      root = Path.join(System.tmp_dir!(), "seamstress_prop_nofiles_#{i}_#{System.unique_integer([:positive])}")
      records = Path.join([root, "seams", "records"])
      File.mkdir_p!(records)
      # Add a decoy file that is NOT a .seam.json
      File.write!(Path.join(records, "README.adoc"), "# placeholder")
      root
    end
  end

  # ---------------------------------------------------------------------------
  # Property invariants
  # ---------------------------------------------------------------------------

  describe "invariant: non-existent paths always raise" do
    test "100 random nonexistent paths all raise RuntimeError" do
      paths = random_nonexistent_paths(100)

      for path <- paths do
        assert_raise RuntimeError, fn ->
          Seamstressd.Runner.validate!(path)
        end
      end
    end

    test "unicode and special-character paths always raise" do
      paths = [
        "/tmp/séam stressd test/path",
        "/tmp/seam\x00stressd",
        "/tmp/seam stressd spaces",
        "/tmp/seamstressd-éàü-#{System.unique_integer([:positive])}"
      ]

      for path <- paths do
        assert_raise RuntimeError, fn ->
          Seamstressd.Runner.validate!(path)
        end
      end
    end
  end

  describe "invariant: empty directories always raise" do
    test "10 empty temp dirs all raise RuntimeError" do
      dirs = empty_temp_dirs(10)

      for dir <- dirs do
        assert_raise RuntimeError, fn ->
          Seamstressd.Runner.validate!(dir)
        end
      end
    end

    test "directories with records/ but no .seam.json files always raise" do
      dirs = dirs_with_records_dir_but_no_files(10)

      for dir <- dirs do
        assert_raise RuntimeError, fn ->
          Seamstressd.Runner.validate!(dir)
        end
      end
    end
  end

  describe "invariant: error messages are always well-formed" do
    test "error message is always a non-empty string" do
      paths = random_nonexistent_paths(20)

      for path <- paths do
        error =
          assert_raise RuntimeError, fn ->
            Seamstressd.Runner.validate!(path)
          end

        assert is_binary(error.message), "error.message must be a binary"
        assert String.length(error.message) > 0, "error.message must be non-empty"
      end
    end

    test "error message always contains the phrase 'exit code'" do
      paths = random_nonexistent_paths(10)

      for path <- paths do
        error =
          assert_raise RuntimeError, fn ->
            Seamstressd.Runner.validate!(path)
          end

        assert error.message =~ "exit code",
               "Expected 'exit code' in: #{inspect(error.message)}"
      end
    end
  end

  describe "invariant: determinism" do
    test "same invalid path produces same error class on repeated calls" do
      path = "/tmp/seamstress_determinism_#{System.unique_integer([:positive])}"

      classes =
        for _ <- 1..5 do
          try do
            Seamstressd.Runner.validate!(path)
          rescue
            e -> e.__struct__
          end
        end

      unique_classes = Enum.uniq(classes)
      assert length(unique_classes) == 1,
             "Expected a single error class across runs, got: #{inspect(unique_classes)}"
    end
  end

  describe "invariant: process safety" do
    test "validate!/1 never kills the calling process" do
      # Spawn a new process for each call so we can detect if it crashes.
      paths = random_nonexistent_paths(10)

      for path <- paths do
        ref = make_ref()
        parent = self()

        spawn(fn ->
          result =
            try do
              Seamstressd.Runner.validate!(path)
              :ok
            rescue
              _e -> :raised
            end

          send(parent, {ref, result})
        end)

        assert_receive {^ref, :raised}, 30_000
      end
    end

    test "validate!/1 does not leak processes" do
      initial_count = Process.list() |> length()
      paths = random_nonexistent_paths(5)

      for path <- paths do
        try do
          Seamstressd.Runner.validate!(path)
        rescue
          _e -> :ok
        end
      end

      # Allow brief settling time, then check process count has not exploded.
      Process.sleep(100)
      final_count = Process.list() |> length()

      # Allow some tolerance for process churn in the test runner itself.
      assert final_count - initial_count < 20,
             "Process leak suspected: #{initial_count} -> #{final_count}"
    end
  end
end
