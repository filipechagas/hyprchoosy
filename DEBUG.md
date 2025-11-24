# Debug Logging

Hyprchoosy includes an optional debug logging feature to help troubleshoot URL routing issues.

## Building with Debug Mode

### Debug Build (with logging)
```bash
cargo build --release --features debug
```

Or use the convenience script:
```bash
./build-debug.sh
```

### Normal Build (without logging)
```bash
cargo build --release
```

## Installing Debug Build

After building with debug mode:
```bash
sudo cp target/release/hyprchoosy /usr/bin/hyprchoosy
```

## Viewing Logs

Debug logs are written to: `/tmp/hyprchoosy/hyprchoosy.log`

View logs in real-time:
```bash
tail -f /tmp/hyprchoosy/hyprchoosy.log
```

View entire log file:
```bash
cat /tmp/hyprchoosy/hyprchoosy.log
```

Clear old logs:
```bash
rm /tmp/hyprchoosy/hyprchoosy.log
```

## What Gets Logged

The debug build logs:

1. **URL Processing**: Every URL that gets passed to hyprchoosy
2. **Client Detection**: Complete process tree walk showing:
   - Each parent process examined
   - Process names and PIDs
   - Which processes are skipped as wrappers
   - Final detected client (e.g., thunderbird, slack, etc.)
3. **Configuration**: Which config sections are checked
4. **Rule Matching**: 
   - Client rule evaluation and matches
   - Host/URL pattern evaluation and matches
5. **Browser Launch**: Which browser is selected and launched
6. **Errors**: Any failures during the process

## Example Log Output

When Thunderbird opens a link, you'll see something like:
```
[INFO] === Hyprchoosy Debug Session Started ===
[INFO] Log file: /tmp/hyprchoosy/hyprchoosy.log
[INFO] === Starting hyprchoosy ===
[INFO] Received URL: 'https://example.com'
[DEBUG] Starting client detection...
[DEBUG] Current PID: 12345
[DEBUG] Step 0: PID 12345 -> PPID 12344 (name: 'xdg-open')
[DEBUG]   Name 'xdg-open' is a wrapper
[DEBUG] Step 1: PID 12344 -> PPID 12343 (name: 'thunderbird')
[DEBUG]   Name 'thunderbird' is NOT a wrapper
[INFO] Detected client: 'thunderbird'
[INFO] Checking client rules for 'thunderbird'
[DEBUG] Matching client: 'thunderbird'
[DEBUG] Available sections: ["work", "personal"]
[DEBUG]   Checking section 'work' with clients: ["thunderbird", "slack"]
[INFO] Client 'thunderbird' matched rule 'work' (pattern: 'thunderbird')
[INFO] Using browser from client rule: 'firefox'
[INFO] Launching browser: 'firefox' with URL: 'https://example.com'
[INFO] Successfully spawned browser 'firefox'
```

## Troubleshooting Thunderbird Issues

To debug why Thunderbird links aren't routing correctly:

1. Build and install debug version:
   ```bash
   ./build-debug.sh
   sudo cp target/release/hyprchoosy /usr/bin/hyprchoosy
   ```

2. Start watching logs:
   ```bash
   tail -f /tmp/hyprchoosy/hyprchoosy.log
   ```

3. Click a link in Thunderbird

4. Check the logs to see:
   - Is Thunderbird being detected as the client?
   - Are your client rules being evaluated?
   - Which browser is being selected?
   - Are there any errors?

## Performance Impact

The debug feature has minimal performance impact because:
- It only writes to a file (no console output)
- Logging code is completely compiled out in normal builds
- File I/O is buffered

However, for production use, it's recommended to use the normal build without debug features.
