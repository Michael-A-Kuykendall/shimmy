#!/bin/bash
# Generate an improved Homebrew formula for Shimmy
# This can be used as reference for updating the official Homebrew formula

echo "🍺 Generating Improved Homebrew Formula for Shimmy"
echo "================================================="

# Get the latest version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "📋 Current version: $VERSION"

# Generate the formula
cat > shimmy.rb << EOF
class Shimmy < Formula
  desc "Lightweight 5MB Ollama alternative with native SafeTensors support"
  homepage "https://github.com/Michael-A-Kuykendall/shimmy"
  version "$VERSION"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v#{version}/shimmy-macos-intel"
      sha256 "TO_BE_CALCULATED"
    else
      url "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v#{version}/shimmy-macos-arm64"
      sha256 "TO_BE_CALCULATED"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v#{version}/shimmy-linux-x86_64"
      sha256 "TO_BE_CALCULATED"
    else
      url "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v#{version}/shimmy-linux-arm64"
      sha256 "TO_BE_CALCULATED"
    end
  end

  def install
    bin.install Dir["*"].first => "shimmy"
    
    # Create shell completions directory
    generate_completions_from_executable(bin/"shimmy", "--help")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/shimmy --version")
    
    # Test that the binary is functional
    output = shell_output("#{bin}/shimmy list 2>&1", 0)
    assert_match "No models", output
  end

  def caveats
    <<~EOS
      Shimmy is now installed! Quick start:

      1. Start the server:
         shimmy serve

      2. List available models:
         shimmy list

      3. Discover models in common locations:
         shimmy discover

      4. Add custom model directories:
         shimmy --model-dirs "path/to/models" serve

      The server will be available at http://127.0.0.1:11434 (Ollama-compatible API)

      For custom model directories, you can also set:
        export SHIMMY_MODEL_PATHS="/path/to/models;/another/path"
        export OLLAMA_MODELS="/path/to/ollama/models"
    EOS
  end
end
EOF

echo "✅ Generated improved Homebrew formula: shimmy.rb"
echo ""
echo "📋 Key improvements in this formula:"
echo "  • Uses pre-built binaries instead of building from source"
echo "  • No Rust/CMake dependencies required"
echo "  • Much faster installation"
echo "  • Platform-specific binaries for Intel and ARM"
echo "  • Includes shell completions"
echo "  • Comprehensive test suite"
echo "  • Helpful caveats with usage instructions"
echo ""
echo "🔧 To use this formula:"
echo "  1. Calculate SHA256 hashes for each release binary"
echo "  2. Update the sha256 fields in the formula"
echo "  3. Submit PR to Homebrew Core repository"
echo ""
echo "💡 For immediate use, users can install via:"
echo "  curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64 -o shimmy"
echo "  chmod +x shimmy && sudo mv shimmy /usr/local/bin/"