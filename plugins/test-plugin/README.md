# Test Plugin

A simple test plugin for the Me-Nexus plugin system to verify functionality.

## Features

- **Ping**: Basic connectivity test
- **Status**: Returns plugin status and metadata

## Usage

This plugin is automatically loaded by the Me-Nexus plugin system when placed in the plugins directory.

## API

### `ping()`
Returns a simple pong response to verify the plugin is working.

### `getStatus()`
Returns detailed status information including:
- Plugin name and version
- Current status (active/inactive)
- Timestamp

## Testing

You can test this plugin directly using Deno:

```bash
deno run index.ts
```

## Configuration

The plugin requires no additional configuration and uses minimal permissions.
