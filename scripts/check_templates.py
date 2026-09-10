#!/usr/bin/env python3
"""Create and compile every bundled template in an isolated temporary workspace."""
from pathlib import Path
import shutil
import os
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
EXE = ".exe" if os.name == "nt" else ""


def main():
    # Ask Cargo for the binary path so CARGO_TARGET_DIR overrides work too.
    import json
    result = subprocess.run(
        ['cargo', 'build', '--message-format=json'], cwd=ROOT,
        check=True, capture_output=True, text=True, encoding="utf-8",
    )
    binaries = [entry['executable'] for line in result.stdout.splitlines()
                if (entry := json.loads(line)).get('executable')]
    binary = binaries[-1]
    with tempfile.TemporaryDirectory(prefix='texsmith-templates-') as directory:
        workspace = Path(directory)
        shipped_binary = workspace / ('texsmith' + EXE)
        shutil.copy2(binary, shipped_binary)
        command = [str(shipped_binary), '--root', directory]
        names = subprocess.check_output(command + ['templates'], text=True, encoding="utf-8").splitlines()
        for name in names:
            subprocess.run(command + ['new', name + '-check', 'Paper & Results',
                                     '--template', name], check=True)
        subprocess.run(command + ['build', '--all'], check=True)
        for name in names:
            report = workspace / (name + '-check')
            pdf = report / 'build/main.pdf'
            assert pdf.read_bytes().startswith(b'%PDF-'), f'{name}: missing PDF'
            log_path = report / 'build/main.log'
            if not log_path.exists():
                # acmart reads its log during compilation; Tectonic then treats
                # it as an intermediate even when --keep-logs is requested.
                subprocess.run([os.environ.get('TECTONIC', 'tectonic'),
                                '--keep-intermediates', '--keep-logs',
                                '--outdir', 'build', 'main.tex'], cwd=report, check=True)
            log = log_path.read_text(encoding="utf-8", errors="replace")
            assert 'undefined' not in log.lower(), f'{name}: unresolved references or fonts in log'
            assert 'Some font shapes were not available' not in log, f'{name}: font substitution'
            for source in report.rglob('*.tex'):
                assert '@@TITLE@@' not in source.read_text(encoding="utf-8"), f'{name}: unresolved title'
            # When available, also verify that BibTeX's example entry reaches the PDF.
            if ((report / 'references.bib').exists() or (report / 'references.tex').exists()) and shutil.which('pdftotext'):
                text = subprocess.check_output(['pdftotext', '-enc', 'UTF-8', str(pdf), '-'], text=True, encoding="utf-8")
                assert 'Knuth' in text, f'{name}: bibliography absent from PDF'
        print(f'All {len(names)} templates compiled successfully with resolved references.')


if __name__ == '__main__':
    main()
