class Shimmy < Formula
  desc "Local model inference server with the default Airframe engine"
  homepage "https://github.com/Michael-A-Kuykendall/shimmy"
  version "2.6.7"
  # Primary components; complete dependency terms are installed under share/shimmy.
  license all_of: ["Apache-2.0", "MIT"]

  on_macos do
    depends_on arch: :arm64
    url "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v2.6.7/shimmy-macos-arm64.tar.gz"
    sha256 "989acb48fa452fd0f7992c1c968752ab8e22772d47ac1770fcd54dc64fbcd694"
  end

  on_linux do
    on_intel do
      url "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v2.6.7/shimmy-linux-x86_64.tar.gz"
      sha256 "ce23fcb71f56e5d84b5848d2f56a5aded40ad37e0a9b5238ce36da487d39c312"
    end
    on_arm do
      url "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v2.6.7/shimmy-linux-arm64.tar.gz"
      sha256 "c560fc3214984deb217eb96859e5b0ccd3a96130032a78963642769f208b229d"
    end
  end

  def install
    odie "Shimmy 2.6.7 has no macOS Intel binary." if OS.mac? && Hardware::CPU.intel?
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
    assert_match "2.6.7", shell_output("#{bin}/shimmy --version")
    assert_match "serve", shell_output("#{bin}/shimmy --help")
    assert_path_exists share/"shimmy/LICENSES/libfse-0.1.4/LICENSE-GRANT-0.1.4.md"
    assert_path_exists share/"shimmy/THIRD-PARTY-NOTICES.md"
  end
end
