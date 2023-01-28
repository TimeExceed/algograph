#!/usr/bin/python3
import subprocess as sp
import os
from pathlib import Path
import shutil as sh

def clean_dir():
    rsdoc_dir = Path('target/doc')
    sh.rmtree(rsdoc_dir)

def cargo_doc():
    env = os.environ.copy()
    env['RUSTDOCFLAGS'] = '--html-in-header ./katex.html'
    sp.check_call(['cargo', 'doc', '--no-deps'], env=env)

if __name__ == '__main__':
    clean_dir()
    cargo_doc()
