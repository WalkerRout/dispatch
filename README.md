# dispatch
Define global keyboard shortcuts to execute shell commands

## Platforms
- ### ✔️ Windows
- ### ❌ Linux
- ### ❌ MacOS

Binary expects that file `dispatch.json` exists in the same directory as the executable, `dispatch.log` will be automatically generated on each run.

## Example `dispatch.json`
> Modifier keys are: 'Ctrl', 'Shift', 'Alt', and 'Super'.
> There is no built in exit key, but you can send `b"shutdown"` to `localhost:33599` to gracefully terminate the application.
```
{
  "keybinds": [
    {
      "keys": ["Ctrl", "Shift", "Alt", "E"],
      "script": "powershell -Command \"echo shutdown | ncat 127.0.0.1 33599\""
    },
    {
      "keys": ["Ctrl", "Shift", "Alt", "T"],
      "script": "PowerShell -Command \"Add-Type -AssemblyName PresentationFramework;[System.Windows.MessageBox]::Show('Daemon Running')\""
    },
    {
      "keys": ["Shift", "Super", "R"],
      "script": "firefox -private-window https://www.rust-lang.org/"
    },
    {
      "keys": ["Shift", "Q", "W", "E"],
      "script": "..."
    }
  ]
}
```
## Bugs
- Adding comments to the `dispatch.json` file will result in an invalid config
- The logic for keypresses is not on press, but on hold, so holding down a combination will run the associated script ~10 times a second
