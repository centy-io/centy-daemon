# Windows: Daemon reports 'already installed' but 'not found' when starting

On Windows, the daemon installation state is inconsistent:

1. Running `centy install daemon` reports:
   - 'Detected platform: x86_64-pc-windows-msvc'
   - 'Error: Daemon already installed at C:\Users\yonib\.centy\bin\centy-daemon.exe. Use --force to reinstall.'

2. But running `centy start` fails with:
   - 'Starting daemon in background...'
   - 'Error: Daemon not found at: centy-daemon'

The daemon file exists according to install but cannot be found by start command. This suggests the start command is looking in the wrong location or using a different path resolution than the install command.
