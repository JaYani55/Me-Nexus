import { readLines } from "https://deno.land/std@0.224.0/io/read_lines.ts";

// Define the message structures on the Deno side
interface RpcRequest {
  id: number;
  method: string;
  params: unknown;
}

interface RpcResponse {
  id: number;
  result?: unknown;
  error?: string;
}

// --- Command Handlers ---
// A simple test handler to verify communication.
async function handlePing(): Promise<string> {
  return "pong";
}

// Get information about the plugin manager
async function handleGetInfo(): Promise<Record<string, unknown>> {
  return {
    version: "1.0.0",
    runtime: "Deno",
    denoVersion: Deno.version.deno,
    typescriptVersion: Deno.version.typescript,
    v8Version: Deno.version.v8,
    capabilities: ["ping", "get_info", "list_plugins", "test_plugin"],
    timestamp: new Date().toISOString(),
  };
}

// List available plugins (for future expansion)
async function handleListPlugins(): Promise<string[]> {
  // In the future, this would scan for available plugin files
  return ["core-ping", "example-plugin"];
}

// Test a specific plugin
async function handleTestPlugin(params: unknown): Promise<Record<string, unknown>> {
  const pluginData = params as { plugin_id?: string };
  const pluginId = pluginData?.plugin_id;
  
  if (!pluginId) {
    throw new Error("plugin_id parameter is required");
  }
  
  // For now, simulate plugin testing
  // In a real implementation, this would load and execute the plugin
  if (pluginId === "test-plugin") {
    return {
      plugin_id: pluginId,
      status: "active",
      message: "Test plugin responded successfully",
      timestamp: new Date().toISOString()
    };
  } else {
    throw new Error(`Plugin '${pluginId}' not found`);
  }
}

// --- Main Loop ---
async function main() {
  console.error("Deno plugin manager starting up...");
  
  for await (const line of readLines(Deno.stdin)) {
    console.error(`[DEBUG] Received line: ${line}`);
    
    try {
      const request: RpcRequest = JSON.parse(line);
      console.error(`[DEBUG] Parsed request: ${JSON.stringify(request)}`);
      
      let result: unknown = null;
      let error: string | undefined = undefined;

      // Route the request to the correct handler
      switch (request.method) {
        case "ping":
          result = await handlePing();
          console.error(`[DEBUG] Ping handled, result: ${result}`);
          break;
        case "get_info":
          result = await handleGetInfo();
          console.error(`[DEBUG] Get info handled, result: ${JSON.stringify(result)}`);
          break;
        case "list_plugins":
          result = await handleListPlugins();
          console.error(`[DEBUG] List plugins handled, result: ${JSON.stringify(result)}`);
          break;
        case "test_plugin":
          result = await handleTestPlugin(request.params);
          console.error(`[DEBUG] Test plugin handled, result: ${JSON.stringify(result)}`);
          break;
        // Future methods like "initialize" or "execute_plugin" go here
        default:
          error = `Unknown method: ${request.method}`;
          console.error(`[DEBUG] Unknown method: ${request.method}`);
      }

      const response: RpcResponse = { id: request.id };
      if (error) {
        response.error = error;
      } else {
        response.result = result;
      }

      // Write the response back to stdout for Rust to read
      const responseJson = JSON.stringify(response);
      console.log(responseJson);
      console.error(`[DEBUG] Sent response: ${responseJson}`);

    } catch (e) {
      const error = e instanceof Error ? e.message : String(e);
      console.error(`[ERROR] Failed to parse request: ${error}`);
      const errorResponse: RpcResponse = {
        id: -1, // No ID if request is unparseable
        error: `Failed to parse request: ${error}`,
      };
      console.log(JSON.stringify(errorResponse));
    }
  }
  
  console.error("Deno plugin manager shutting down...");
}

main().catch((e) => {
  const error = e instanceof Error ? e.message : String(e);
  console.error(`[FATAL] Main loop failed: ${error}`);
  Deno.exit(1);
});
