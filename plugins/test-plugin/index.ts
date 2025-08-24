// Test Plugin for Me-Nexus Plugin System
// This plugin demonstrates basic plugin functionality

export interface PluginAPI {
  ping(): Promise<string>;
  getStatus(): Promise<{ 
    name: string; 
    version: string; 
    status: "active" | "inactive"; 
    timestamp: string;
  }>;
}

class TestPlugin implements PluginAPI {
  private name = "Test Plugin";
  private version = "1.0.0";

  async ping(): Promise<string> {
    return `Pong from ${this.name} v${this.version}`;
  }

  async getStatus(): Promise<{ 
    name: string; 
    version: string; 
    status: "active" | "inactive"; 
    timestamp: string;
  }> {
    return {
      name: this.name,
      version: this.version,
      status: "active",
      timestamp: new Date().toISOString()
    };
  }
}

// Plugin entry point
export default function createPlugin(): PluginAPI {
  return new TestPlugin();
}

// For direct execution testing
if (import.meta.main) {
  const plugin = createPlugin();
  console.log("Test Plugin initialized");
  
  // Test ping functionality
  plugin.ping().then(result => {
    console.log("Ping result:", result);
  });
  
  // Test status functionality
  plugin.getStatus().then(status => {
    console.log("Status:", JSON.stringify(status, null, 2));
  });
}
