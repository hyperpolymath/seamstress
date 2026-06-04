# SPDX-License-Identifier: MPL-2.0
# Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
defmodule Seamstressd.Application do
  @moduledoc false

  use Application

  def start(_type, _args) do
    children = []
    opts = [strategy: :one_for_one, name: Seamstressd.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
