class Shimmy < Formula
  desc "Local model inference server with the default Airframe engine"
  homepage "https://github.com/Michael-A-Kuykendall/shimmy"
  version "2.6.6"
  # Primary components; complete dependency terms are installed under share/shimmy.
  license all_of: ["Apache-2.0", "MIT"]

  on_macos do
    on_arm do
      url "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v2.6.6/shimmy-macos-arm64.tar.gz"
      sha256 "eabba8acf94450242f0b8da221a391a935185b3c3877f1350b1ff7ffec2d4a2e"
    end
    on_intel do
      odie "Shimmy 2.6.6 has no macOS Intel binary; use an independently validated source build."
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v2.6.6/shimmy-linux-x86_64.tar.gz"
      sha256 "13c5ea0b7c32e7021542a94ad961347b6b2a86deb834a1652f6e93e0e7204cf9"
    end
    on_arm do
      url "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v2.6.6/shimmy-linux-arm64.tar.gz"
      sha256 "9916f1279d1a2d3eced746836fecbbe277f4816d21e06da45ebc6596f22f812e"
    end
  end

  def install
    bin.install "shimmy"
    (share/"shimmy").install "LICENSES", "THIRD-PARTY-NOTICES.md", "DISTRIBUTION-LICENSE.txt", "native-manifest.json"
  end

  def caveats
    <<~EOS
      Component and third-party licenses, including the additional Apache-2.0
      grant for libfse 0.1.4, are installed under #{share}/shimmy.
      Native CLI validation is not Metal/M4, DGX or general GPU certification.
      See https://github.com/Michael-A-Kuykendall/shimmy/blob/main/docs/SUPPORTED_MODELS.md
    EOS
  end

  test do
    assert_match "2.6.6", shell_output("#{bin}/shimmy --version")
    assert_match "serve", shell_output("#{bin}/shimmy --help")
    assert_path_exists share/"shimmy/LICENSES/libfse-0.1.4/LICENSE-GRANT-0.1.4.md"
    assert_path_exists share/"shimmy/THIRD-PARTY-NOTICES.md"
  end
end
