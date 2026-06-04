# SPDX-License-Identifier: MPL-2.0
# Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
defmodule SeamstressdTest do
  use ExUnit.Case

  test "runner module loads" do
    assert function_exported?(Seamstressd.Runner, :validate!, 1)
  end
end
