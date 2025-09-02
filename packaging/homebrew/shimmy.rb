class Shimmy < Formula
  desc "The 5MB alternative to Ollama - local AI inference server"
  homepage "https://github.com/Michael-A-Kuykendall/shimmy"
  url "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/vVERSION_PLACEHOLDER/shimmy-VERSION_PLACEHOLDER-darwin-amd64.tar.gz"
  sha256 "SHA256_AMD64_PLACEHOLDER"
  license "MIT"
  version "VERSION_PLACEHOLDER"

  on_arm do
    url "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/vVERSION_PLACEHOLDER/shimmy-VERSION_PLACEHOLDER-darwin-arm64.tar.gz"
    sha256 "SHA256_ARM64_PLACEHOLDER"
  end

  def install
    bin.install "shimmy"
  end

  test do
    system "#{bin}/shimmy", "--version"
    system "#{bin}/shimmy", "list"
  end

  service do
    run [opt_bin/"shimmy", "serve", "--bind", "127.0.0.1:11435"]
    keep_alive true
    log_path var/"log/shimmy.log"
    error_log_path var/"log/shimmy.error.log"
  end
end
