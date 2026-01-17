defmodule Seamstressd.Runner do
  @moduledoc """
  Minimal runner for invoking seamctl.
  """

  def validate!(root) do
    cmd = "cargo"
    args = ["run", "--quiet", "--manifest-path", "tools/seamctl/Cargo.toml", "--", "validate", "--root", root]

    {out, code} = System.cmd(cmd, args, stderr_to_stdout: true)

    IO.write(out)

    if code != 0 do
      raise "seamctl validate failed with exit code #{code}"
    end

    :ok
  end
end
