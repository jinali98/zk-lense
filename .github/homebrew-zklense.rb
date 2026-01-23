# This file is a template for a Homebrew formula
# To use this formula:
# 1. Copy this file to: gihanrcg/homebrew-zklense/Formula/zklense.rb
# 2. Update the URL and SHA256 with the latest release
# 3. Commit and push to your tap repository: https://github.com/gihanrcg/homebrew-zklense
# 
# Users can then install with:
#   brew tap gihanrcg/zklense
#   brew install zklense

class Zklense < Formula
  desc "A command-line tool for profiling, building, and deploying zero-knowledge proofs built with Noir for Solana Blockchain"
  homepage "https://github.com/jinali98/zk-profiling-solana"
  url "https://github.com/jinali98/zk-profiling-solana/releases/download/v0.1.0/zklense-x86_64-apple-darwin.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  version "0.1.0"

  # Determine the correct binary based on architecture
  if Hardware::CPU.intel?
    url "https://github.com/jinali98/zk-profiling-solana/releases/download/v0.1.0/zklense-x86_64-apple-darwin.tar.gz"
    sha256 "REPLACE_WITH_INTEL_SHA256"
  else
    url "https://github.com/jinali98/zk-profiling-solana/releases/download/v0.1.0/zklense-aarch64-apple-darwin.tar.gz"
    sha256 "REPLACE_WITH_ARM_SHA256"
  end

  def install
    bin.install "zklense"
  end

  test do
    system "#{bin}/zklense", "--version"
  end
end
