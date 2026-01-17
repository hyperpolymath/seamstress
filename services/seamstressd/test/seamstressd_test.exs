defmodule SeamstressdTest do
  use ExUnit.Case

  test "runner module loads" do
    assert function_exported?(Seamstressd.Runner, :validate!, 1)
  end
end
