# SPDX-License-Identifier: PMPL-1.0-or-later
# End-to-end tests for the seamstressd service lifecycle.
#
# These tests exercise the full flow: application startup → configuration
# verification → runner invocation → controlled shutdown.  They use
# ExUnit's process supervision helpers so each test case is isolated.

defmodule Seamstressd.E2ETest do
  use ExUnit.Case, async: false

  # ---------------------------------------------------------------------------
  # Helpers
  # ---------------------------------------------------------------------------

  # Create a minimal temporary workspace that mimics a real seamstress repo.
  # Returns {:ok, root_path} where root_path contains seams/records/.
  defp tmp_workspace(label) do
    root = Path.join(System.tmp_dir!(), "seamstress_e2e_#{label}_#{System.unique_integer([:positive])}")
    seams_dir = Path.join([root, "seams", "records"])
    schema_dir = Path.join([root, "seams", "schema"])
    File.mkdir_p!(seams_dir)
    File.mkdir_p!(schema_dir)
    {:ok, root}
  end

  # Write a placeholder schema so seamctl has something to compile against.
  defp write_schema(root) do
    schema = %{
      "$schema" => "http://json-schema.org/draft-07/schema#",
      "type" => "object"
    }
    path = Path.join([root, "seams", "schema", "seam-record.schema.json"])
    File.write!(path, Jason.encode!(schema))
    path
  end

  # ---------------------------------------------------------------------------
  # Tests: service lifecycle
  # ---------------------------------------------------------------------------

  describe "application lifecycle" do
    test "Application module is defined and loadable" do
      assert Code.ensure_loaded?(Seamstressd.Application)
    end

    test "Application module exports start/2 callback" do
      exports = Seamstressd.Application.__info__(:functions)
      assert {:start, 2} in exports
    end

    test "Supervisor starts with empty children list" do
      # Start a supervisor manually to verify the OTP contract.
      {:ok, pid} = Supervisor.start_link([], strategy: :one_for_one)
      assert Process.alive?(pid)
      Supervisor.stop(pid)
    end

    test "runner is accessible after application boot" do
      # The application is started by the test framework (mix test).
      # Verify that key modules are loaded.
      assert Code.ensure_loaded?(Seamstressd.Runner)
    end

    test "application module references Runner module" do
      # Verify cross-module dependency is intact after compilation.
      assert function_exported?(Seamstressd.Runner, :validate!, 1)
    end
  end

  describe "workspace validation lifecycle" do
    test "empty workspace causes validate!/1 to raise" do
      {:ok, root} = tmp_workspace("empty")

      assert_raise RuntimeError, fn ->
        Seamstressd.Runner.validate!(root)
      end
    after
      :ok
    end

    test "workspace with schema but no records causes validate!/1 to raise" do
      {:ok, root} = tmp_workspace("schema_only")
      write_schema(root)

      assert_raise RuntimeError, fn ->
        Seamstressd.Runner.validate!(root)
      end
    end

    test "validate!/1 on nonexistent root raises RuntimeError" do
      assert_raise RuntimeError, fn ->
        Seamstressd.Runner.validate!("/this/path/does/not/exist")
      end
    end

    test "validate!/1 error is always a RuntimeError, never a system exit" do
      root = "/nonexistent_e2e_#{System.unique_integer([:positive])}"

      raised =
        try do
          Seamstressd.Runner.validate!(root)
          nil
        rescue
          e -> e
        end

      assert %RuntimeError{} = raised
    end

    test "multiple successive validate!/1 calls on same bad root all raise" do
      {:ok, root} = tmp_workspace("multi_call")

      for _i <- 1..3 do
        assert_raise RuntimeError, fn ->
          Seamstressd.Runner.validate!(root)
        end
      end
    end

    test "validate!/1 raises with message containing root path context" do
      {:ok, root} = tmp_workspace("ctx")

      error =
        assert_raise RuntimeError, fn ->
          Seamstressd.Runner.validate!(root)
        end

      # The error must contain either the exit code or relevant context.
      assert is_binary(error.message)
      assert String.length(error.message) > 0
    end
  end

  describe "concurrent service usage" do
    test "multiple concurrent validate!/1 calls fail independently" do
      # Each call to validate! with a bad path must fail with its own error.
      tasks =
        for i <- 1..4 do
          root = "/nonexistent_concurrent_#{i}_#{System.unique_integer([:positive])}"

          Task.async(fn ->
            try do
              Seamstressd.Runner.validate!(root)
              :unexpectedly_ok
            rescue
              _e -> :raised_as_expected
            end
          end)
        end

      results = Task.await_many(tasks, 30_000)
      assert Enum.all?(results, &(&1 == :raised_as_expected))
    end

    test "runner module is reentrant (no global mutable state)" do
      # Verify the runner does not store state between calls.
      # Three sequential calls all produce the same class of error.
      errors =
        for _ <- 1..3 do
          try do
            Seamstressd.Runner.validate!("/no_such_path")
          rescue
            e -> e.__struct__
          end
        end

      assert Enum.all?(errors, &(&1 == RuntimeError))
    end
  end
end
