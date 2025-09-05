# HiveTechs Consensus Homebrew Formula
class Hive < Formula
  desc "AI-powered codebase intelligence platform with multi-model consensus"
  homepage "https://hive.ai"
  url "https://github.com/hivetechs/hive/releases/download/v2.0.0/hive-macos-universal.tar.gz"
  version "2.0.0"
  sha256 "PLACEHOLDER_SHA256"
  license :cannot_represent

  depends_on "curl"
  depends_on "git" => :optional

  def install
    bin.install "hive"
    
    # Install shell completions
    generate_completions_from_executable(bin/"hive", "completions")
    
    # Install man page
    man1.install "docs/hive.1" if File.exist?("docs/hive.1")
  end

  def post_install
    puts <<~EOS
      🐝 HiveTechs Consensus installed successfully!
      
      Get started:
        hive --help         # Show available commands
        hive setup          # Configure API keys
        hive ask "Hello"    # Test consensus engine
      
      Documentation: https://hive.ai/docs
      Support: https://github.com/hivetechs/hive/issues
    EOS
  end

  test do
    system "#{bin}/hive", "--version"
    system "#{bin}/hive", "--help"
  end

  def caveats
    <<~EOS
      To get started with HiveTechs Consensus:
      
      1. Configure your API keys:
         hive setup
      
      2. Test the installation:
         hive ask "Hello world"
      
      3. Explore repository intelligence:
         cd your-project && hive analyze .
      
      For more information, visit https://hive.ai/docs
    EOS
  end
end