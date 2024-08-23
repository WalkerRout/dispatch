# dispatch
Define global keyboard shortcuts to execute shell commands

## Platforms
- ### ✔️ Windows
- ### ❌ Linux
- ### ❌ MacOS

Binary expects that file `dispatch.json` exists in the same directory as the executable, `dispatch.log` will be automatically generated on each run.

## Example `dispatch.json`
> Modifier keys are: 'Ctrl', 'Shift', 'Alt', and 'Super'
```
{
  "keybinds": [
    // test if daemon is running
    {
      "keys": ["Ctrl", "Shift", "Alt", "T"],
      "script": "PowerShell -Command \"Add-Type -AssemblyName PresentationFramework;[System.Windows.MessageBox]::Show('Daemon Running')\""
    },
    // launch firefox
    {
      "keys": ["Shift", "Super", "R"],
      "script": "firefox -private-window https://www.rust-lang.org/"
    },
    // do something else...
    {
      "keys": ["Shift", "Q", "W", "E"],
      "script": "..."
    }
  ]
}
```