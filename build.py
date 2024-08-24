import zipfile
import os
import platform
import subprocess

app_name = "motd"

targets = {
    "Windows": {
        "x86_64-pc-windows-msvc": "windows-x64",
        "i686-pc-windows-msvc": "windows-x86",
        "aarch64-pc-windows-msvc": "windows-arm64"
    },
    "Linux": {
        "x86_64-unknown-linux-gnu": "linux-x64",
        "aarch64-unknown-linux-gnu": "linux-arm64",
        "i686-unknown-linux-gnu": "linux-x86"
    },
    "Darwin": {
        "x86_64-apple-darwin": "macos-intel",
        "aarch64-apple-darwin": "macos-apple-silicon"
    }
}

os.environ["RUSTFLAGS"] = "-C target-feature=+crt-static"

os_type = platform.system()
os.makedirs("dist", exist_ok=True)

for target, alias in targets[os_type].items():
    if os_type == "Linux":
        subprocess.Popen(f"sudo apt install gcc-aarch64-linux-gnu -y", stdout=subprocess.PIPE, text=True, shell=True).wait()
        subprocess.Popen(f"sudo apt install gcc-i686-linux-gnu -y", stdout=subprocess.PIPE, text=True, shell=True).wait()
    subprocess.Popen(f"rustup target add {target}", stdout=subprocess.PIPE, text=True, shell=True).wait()
    subprocess.Popen(f"cargo build -r --target {target}", stdout=subprocess.PIPE, text=True, shell=True, env=os.environ).wait()
    with zipfile.ZipFile(os.path.join("dist", f"{app_name}-{alias}.zip"), "w", zipfile.ZIP_DEFLATED) as zipf:
        if os_type == "Windows":
            app_name_with_extension = f"{app_name}.exe"
        else:
            app_name_with_extension = app_name
        zipf.write(os.path.join("target", target, "release", app_name_with_extension), arcname=app_name_with_extension)