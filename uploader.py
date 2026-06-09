import io
import zstandard as zstd
import tarfile
import requests
import questionary

def main():
    target_repo = None
    
    while True:
        action = questionary.select(
            "What do you want to do?",
            choices=["Set Target Repository", "Upload new", "Exit"]
        ).ask()

        if action == "Set Target Repository":
            url = questionary.text("Enter repository URL:").ask()
            token = questionary.text("Enter auth token:").ask()
            target_repo = {"url": url, "token": token}
            print("Repository settings saved.")

        elif action == "Upload new":
            if not target_repo:
                print("Error: Set target repository first!")
                continue

            name = questionary.text("Package name:").ask()
            pkg_dir = questionary.path("Path to package directory:").ask()
            version = questionary.text("Version:").ask()
            description = questionary.text("Description:").ask()
            
            mode = questionary.select(
                "Is your package source based?",
                choices=["Source only", "Binary only", "Both"]
            ).ask()
            source_based = mode in ["Source only", "Both"]
            binary_based = mode in ["Binary only", "Both"]
            
            build_cmd = questionary.text("Build command (optional):").ask() if source_based else ""
            raw_deps = questionary.text("Dependencies (comma separated, leave blank for none):").ask()
            if raw_deps.strip():
                deps = [d.strip() for d in raw_deps.split(",")]
            else:
                deps = []

            print("Compressing...")
            buffer = io.BytesIO()
            cctx = zstd.ZstdCompressor(level=19)

            class NonClosingBytesIO:
                def __init__(self, stream):
                    self.stream = stream
                def write(self, data):
                    return self.stream.write(data)
                def close(self):
                    pass 

            with cctx.stream_writer(NonClosingBytesIO(buffer)) as z_writer:
                with tarfile.open(fileobj=z_writer, mode="w") as tar:
                    tar.add(pkg_dir, arcname=".")

            buffer.seek(0)
            compressed_data = buffer.getvalue()
            
            print(f"Uploading {name} to {target_repo['url']}...")
            
            files = {'file': (f"{name}.tar.zst", compressed_data)}
            data = {
                'name': name,
                'version': version,
                'description': description,
                'source_based': str(source_based),
                'binary_based': str(binary_based),
                'build_cmd': build_cmd,
                'dependencies': deps
            }
            
            try:
                resp = requests.post(
                    f"{target_repo['url']}/upload",
                    headers={"x-fpkg-upload-token": target_repo['token']},
                    data=data,
                    files=files
                )
                resp.raise_for_status()
                print(f"Success: {resp.text}")
            except requests.exceptions.RequestException as e:
                print(f"Upload failed: {e}")

        elif action == "Exit":
            break

if __name__ == "__main__":
    main()