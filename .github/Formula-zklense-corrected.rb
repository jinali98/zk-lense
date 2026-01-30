class Zklense < Formula
  desc "A command-line tool for profiling, building, and deploying zero-knowledge proofs built with Noir for Solana Blockchain"
  homepage "https://github.com/jinali98/zk-lense"
  version "0.1.1"
  license "MIT OR Apache-2.0"

  if Hardware::CPU.intel?
    url "https://github.com/jinali98/zk-lense/releases/download/v0.1.1/zklense-x86_64-apple-darwin.tar.gz"
    sha256 "a87a118ce01bed502ffa934ce4d1dd66a453a8f47afd004539c38f96913b2a82"
  else
    url "https://github.com/jinali98/zk-lense/releases/download/v0.1.1/zklense-aarch64-apple-darwin.tar.gz"
    sha256 "bbc9283c9777c02421672f5af0966f413d3327614c177c1c7e9d09fdd7e42eeb"
  end

  def install
    bin.install "zklense"
  end

  test do
    system "#{bin}/zklense", "--version"
  end
end
