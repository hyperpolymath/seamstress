defmodule Seamstressd.MixProject do
  use Mix.Project

  def project do
    [
      app: :seamstressd,
      version: "0.1.0",
      elixir: "~> 1.16",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      aliases: aliases()
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {Seamstressd.Application, []}
    ]
  end

  defp deps do
    []
  end

  defp aliases do
    [
      "seam.validate": ["run -e 'Seamstressd.Runner.validate!(".")'"]
    ]
  end
end
