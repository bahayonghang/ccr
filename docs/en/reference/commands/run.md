# run - Temporary Run Command

Run an AI CLI tool (e.g., Claude Code, Codex) or any other command temporarily in an isolated child process using the environment variables of a specified configuration, without modifying the global or default configuration files.

## Usage

```bash
ccr run <config_name> -- <command_and_args>
```

## Parameters

- `<config_name>`: The name of the configuration to use (required).
- `<command_and_args>`: The command and arguments after `[--]` will be executed as a child process.

## Execution Flow

1. 🔍 **Find Platform by Config**: Locate the specified configuration across all platforms.
2. 🧮 **Prepare Isolated Env Variables**: Extract the environment variables (e.g., `ANTHROPIC_API_KEY`) corresponding to this config, and print them automatically with masking.
3. 🚀 **Execute Command**: Launch the child process using the extracted environment variables, forwarding standard IO.

## Examples

```bash
# Temporarily run Claude Code with the 'anyrouter' config, printing Hello
ccr run anyrouter -- claude -p "Hello"

# Temporarily run Codex using the 'my_codex' config
ccr run my_codex -- codex
```

## Core Advantages

- **No Side Effects**: Unlike `switch` which modifies the global `settings.json`, `run` isolates the environment by injecting environment variables directly into the child process.
- **Concurrency Safe**: You can run Claude Code with different LLM configurations simultaneously in multiple terminal windows without interference.
- **Secure Masking**: Sensitive information (like tokens or keys) is masked when printing the injected environment variables.

## Error Handling

### Configuration Not Found

If the specified `config_name` cannot be found in any platform, an error will be displayed:

```bash
$ ccr run nonexistent -- claude
Error: 在所有平台中均未找到配置 'nonexistent'
💡 提示:
  • 运行 'ccr list' 查看可用配置
```

## Related Commands

- [switch](./switch) - Switch configuration globally
- [list](./list) - List available configurations
- [current](./current) - View current configuration
