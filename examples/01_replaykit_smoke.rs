use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use replaykit::ScreenRecorder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ReplayKit requires a bundle context on macOS — relaunch inside a minimal
    // .app bundle if we haven't done so already.
    if env::var_os("REPLAYKIT_SMOKE_BUNDLED").is_none() {
        return relaunch_inside_app_bundle();
    }

    let recorder = ScreenRecorder::shared()
        .ok_or("RPScreenRecorder.shared() returned nil — is ReplayKit available?")?;

    let available = recorder.is_available();
    println!("ScreenRecorder::shared()?.is_available = {available}");
    println!("✅ replaykit recorder OK");
    Ok(())
}

fn relaunch_inside_app_bundle() -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = env::current_exe()?;
    let crate_root = env::current_dir()?;
    let app_root = crate_root.join("target/replaykit-smoke.app");
    let contents_dir = app_root.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let bundle_exe = macos_dir.join(executable_name(&current_exe));

    fs::create_dir_all(&macos_dir)?;
    fs::copy(&current_exe, &bundle_exe)?;
    fs::set_permissions(&bundle_exe, fs::metadata(&current_exe)?.permissions())?;
    fs::write(contents_dir.join("Info.plist"), info_plist())?;

    let status = Command::new(&bundle_exe)
        .env("REPLAYKIT_SMOKE_BUNDLED", "1")
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("bundled smoke runner exited with status {status}").into())
    }
}

fn executable_name(path: &Path) -> String {
    path.file_name()
        .and_then(|v| v.to_str())
        .map_or_else(|| "01_replaykit_smoke".to_owned(), ToOwned::to_owned)
}

fn info_plist() -> String {
    [
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
        "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">",
        "<plist version=\"1.0\">",
        "<dict>",
        "  <key>CFBundleExecutable</key>",
        "  <string>01_replaykit_smoke</string>",
        "  <key>CFBundleIdentifier</key>",
        "  <string>fish.doom.replaykit.smoke</string>",
        "  <key>CFBundleName</key>",
        "  <string>replaykit-smoke</string>",
        "  <key>CFBundlePackageType</key>",
        "  <string>APPL</string>",
        "  <key>LSMinimumSystemVersion</key>",
        "  <string>11.0</string>",
        "</dict>",
        "</plist>",
    ]
    .join("\n")
}
