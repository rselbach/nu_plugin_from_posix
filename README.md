# nu_plugin_from_posix

A Nushell plugin to convert POSIX-style `export` statements to Nushell `$env` assignments.

## Building

```bash
cargo build --release
```

## Installing

After building, register the plugin with Nushell:

```bash
plugin add target/release/nu_plugin_from_posix
```

## Usage

The plugin provides a `from posix` command that can be used in pipelines:

```nushell
# Single export
'export FOO=bar' | from posix
# Output: $env.FOO = bar

# Multiple exports on one line
'export FOO=bar && export BAZ=qux' | from posix
# Output: $env.FOO = bar
#         $env.BAZ = qux

# Multiple variables in one export
'export FOO=bar BAZ=qux' | from posix
# Output: $env.FOO = bar
#         $env.BAZ = qux

# Quoted values
'export PATH="/usr/bin:/bin"' | from posix
# Output: $env.PATH = "/usr/bin:/bin"

# Multiline input
"export FOO=bar
export BAZ=qux" | from posix
# Output: $env.FOO = bar
#         $env.BAZ = qux
```

## Features

- Handles single and multiple export statements
- Supports `&&` separated commands on the same line
- Properly parses quoted values (both single and double quotes)
- Handles escape sequences in double-quoted values
- Converts multiline input with multiple export statements