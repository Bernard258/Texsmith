#!/usr/bin/env python3
"""Verify automatic discovery, incremental rebuilds, and single-binary delivery."""
from pathlib import Path
import os
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
EXE = ".exe" if os.name == "nt" else ""


def main():
    with tempfile.TemporaryDirectory(prefix='texsmith-embedding-') as directory:
        temp = Path(directory)
        source = temp / 'source'
        source.mkdir()
        for name in ['Cargo.toml', 'Cargo.lock', 'build.rs']:
            shutil.copy2(ROOT / name, source / name)
        for name in ['src', 'templates']:
            shutil.copytree(ROOT / name, source / name)
        target = temp / 'target'
        environment = dict(os.environ, CARGO_TARGET_DIR=str(target))

        def build():
            subprocess.run(['cargo', 'build', '--quiet'], cwd=source, env=environment, check=True)
            return target / 'debug' / ('texsmith' + EXE)

        binary = build()
        added = source / 'templates/new-format'
        (added / 'sections').mkdir(parents=True)
        (added / 'main.tex').write_text('First: @@TITLE@@', encoding='utf-8')
        (added / 'sections/body.tex').write_text('Body: @@TITLE@@', encoding='utf-8')
        (added / 'asset.bin').write_bytes(bytes([0, 255, 128]))
        binary = build()
        assert 'new-format' in subprocess.check_output([binary, 'templates'], text=True, encoding="utf-8")
        (added / 'main.tex').write_text('Updated: @@TITLE@@', encoding='utf-8')
        binary = build()

        # Ship just the binary, then remove all source templates before using it.
        shipped = temp / ('texsmith' + EXE)
        shutil.copy2(binary, shipped)
        shutil.rmtree(source / 'templates')
        workspace = temp / 'workspace'
        workspace.mkdir()
        command = [str(shipped), '--root', str(workspace)]
        subprocess.run(command + ['new', 'paper', 'A&B', '--template', 'new-format'], check=True)
        assert (workspace / 'paper/main.tex').read_text(encoding="utf-8") == r'Updated: A\&B'
        assert (workspace / 'paper/sections/body.tex').read_text(encoding="utf-8") == r'Body: A\&B'
        assert (workspace / 'paper/asset.bin').read_bytes() == bytes([0, 255, 128])

        # Restoring the original source removes the added template on rebuild.
        shutil.copytree(ROOT / 'templates', source / 'templates')
        binary = build()
        assert 'new-format' not in subprocess.check_output([binary, 'templates'], text=True, encoding="utf-8")
        print('Discovery, updates, removal, binary assets, and standalone delivery passed.')


if __name__ == '__main__':
    main()
