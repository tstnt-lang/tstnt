# TSTNT VSCode syntax highlighting

## Install manually

1. Copy folder to `~/.vscode/extensions/tstnt-lang/`
2. Add to `package.json`:
```json
{
  "name": "tstnt-lang",
  "contributes": {
    "languages": [{"id": "tstnt", "extensions": [".tstnt"]}],
    "grammars": [{"language": "tstnt", "scopeName": "source.tstnt", "path": "./tstnt.tmLanguage.json"}]
  }
}
```
3. Restart VSCode
