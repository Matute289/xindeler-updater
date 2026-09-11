#!/usr/bin/env python3
"""Builds updater-latest.json from the packaged dist/ artifacts.

Same {version, platforms: [{os, arch, file, size, sha256, signed}]} shape as
downloads.xindeler.com/latest.json (xindeler-new-horizon's own manifest) - reusing
the schema xindeler-web-api already knows how to parse, per NH-145.
"""

import hashlib
import json
import os
import re
import sys

# (filename regex, os, arch, whether this platform can be code-signed)
PATTERNS = [
    (r"^xindeler-updater-linux-x86_64\.tar\.gz$", "linux", "x86_64", False),
    (r"^xindeler-updater-linux-aarch64\.tar\.gz$", "linux", "aarch64", False),
    (r"^xindeler-updater-macos-x86_64(-unsigned)?\.zip$", "macos", "x86_64", True),
    (r"^xindeler-updater-macos-aarch64(-unsigned)?\.zip$", "macos", "aarch64", True),
    (
        r"^xindeler-updater-windows-x86_64-installer(-unsigned)?\.exe$",
        "windows",
        "x86_64",
        True,
    ),
]


def sha256_of(path):
    digest = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main():
    if len(sys.argv) != 4:
        print(
            f"usage: {sys.argv[0]} <version> <dist-dir> <output-path>",
            file=sys.stderr,
        )
        sys.exit(1)

    version, dist_dir, output_path = sys.argv[1], sys.argv[2], sys.argv[3]

    platforms = []
    matched_files = set()
    for filename in sorted(os.listdir(dist_dir)):
        for pattern, os_name, arch, can_sign in PATTERNS:
            if re.match(pattern, filename):
                path = os.path.join(dist_dir, filename)
                signed = ("-unsigned" not in filename) if can_sign else None
                platforms.append(
                    {
                        "os": os_name,
                        "arch": arch,
                        "file": filename,
                        "size": os.path.getsize(path),
                        "sha256": sha256_of(path),
                        "signed": signed,
                    }
                )
                matched_files.add(filename)
                break

    expected = len(PATTERNS)
    if len(platforms) != expected:
        unmatched = sorted(set(os.listdir(dist_dir)) - matched_files)
        print(
            f"::error::Expected {expected} platform artifacts, matched "
            f"{len(platforms)}. Unmatched files in {dist_dir}: {unmatched}",
            file=sys.stderr,
        )
        sys.exit(1)

    manifest = {"version": version, "platforms": platforms}
    with open(output_path, "w") as f:
        json.dump(manifest, f, indent=2)
        f.write("\n")

    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    main()
