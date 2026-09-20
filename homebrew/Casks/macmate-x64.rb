cask "macmate-x64" do
  version "0.3.0"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  url "https://github.com/gh-pixel-95813/MacMate/releases/download/v#{version}/MacMate_#{version}_x64.dmg"
  name "MacMate"
  desc "Free, open-source macOS cleaner built with Tauri"
  homepage "https://github.com/gh-pixel-95813/MacMate"
  livecheck do
    url :url
    strategy :github_latest
  end
  depends_on arch: :x86_64
  app "MacMate.app"
  zap trash: [
    "~/Library/Preferences/com.macmate.app.plist",
    "~/Library/Application Support/com.macmate.app",
  ]
end
