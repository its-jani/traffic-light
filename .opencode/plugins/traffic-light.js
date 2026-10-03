import { tool } from "@opencode-ai/plugin";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";

async function isDaemonRunning() {
  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 350);
    const res = await fetch("http://127.0.0.1:8765/ping", { signal: controller.signal });
    clearTimeout(timeout);
    return res.ok;
  } catch {
    return false;
  }
}

async function ensureDaemonRunning() {
  if (await isDaemonRunning()) return true;

  const candidates = [
    "traffic-light",
    path.join(os.homedir(), ".cargo", "bin", "traffic-light.exe"),
    path.join(process.cwd(), "target", "release", "traffic-light.exe"),
    path.join(process.cwd(), "target", "debug", "traffic-light.exe"),
    "d:\\traffic-light\\target\\release\\traffic-light.exe",
  ];

  for (const bin of candidates) {
    try {
      if (bin.includes(path.sep) && !fs.existsSync(bin)) continue;
      const child = spawn(bin, [], {
        detached: true,
        stdio: "ignore",
        windowsHide: false,
      });
      child.unref();

      // Poll until port 8765 is responsive
      for (let i = 0; i < 15; i++) {
        await new Promise((r) => setTimeout(r, 100));
        if (await isDaemonRunning()) return true;
      }
    } catch {
      // try next candidate
    }
  }
  return false;
}

async function sendTrafficPayload(endpoint, body, autoStart = false) {
  try {
    if (autoStart) {
      await ensureDaemonRunning();
    }
    const res = await fetch(`http://127.0.0.1:8765${endpoint}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    return res.ok;
  } catch (e) {
    if (!autoStart && (await ensureDaemonRunning())) {
      try {
        const retryRes = await fetch(`http://127.0.0.1:8765${endpoint}`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(body),
        });
        return retryRes.ok;
      } catch {
        return false;
      }
    }
    return false;
  }
}

let isSessionActive = true;
let activeSessionId = "opencode-agent";

export const TrafficLightPlugin = async (_ctx) => {
  return {
    tool: {
      traffic: tool({
        description: "Control floating desktop traffic light (on, off, green, yellow, red)",
        args: {
          action: tool.schema.enum(["on", "off", "green", "yellow", "red"]).describe("Action to perform"),
          message: tool.schema.string().optional().describe("Status message to display"),
        },
        async execute(args) {
          if (args.action === "on") {
            isSessionActive = true;
            const ok = await sendTrafficPayload(
              "/session/on",
              {
                session_id: activeSessionId,
                label: "OpenCode",
                state: "green",
              },
              true
            );
            return ok
              ? "Traffic Light activated (Green) 🟢"
              : "Failed to connect to Traffic Light daemon. Ensure traffic-light is installed.";
          }
          if (args.action === "off") {
            isSessionActive = false;
            await sendTrafficPayload("/session/off", {
              session_id: activeSessionId,
            });
            return "Traffic Light dismissed ⚪";
          }
          if (isSessionActive) {
            await sendTrafficPayload("/state", {
              session_id: activeSessionId,
              state: args.action,
              message: args.message || "",
            });
          }
          return `Traffic Light state set to ${args.action}`;
        },
      }),
    },

    "chat.message": async (input) => {
      if (input.sessionID) activeSessionId = `opencode-${input.sessionID.slice(0, 8)}`;
      if (isSessionActive) {
        await sendTrafficPayload("/state", {
          session_id: activeSessionId,
          state: "yellow",
          message: "Thinking...",
        });
      }
    },

    "tool.execute.before": async (input) => {
      if (input.sessionID) activeSessionId = `opencode-${input.sessionID.slice(0, 8)}`;
      if (isSessionActive) {
        await sendTrafficPayload("/state", {
          session_id: activeSessionId,
          state: "yellow",
          message: `Running ${input.tool}...`,
        });
      }
    },

    "permission.ask": async () => {
      if (isSessionActive) {
        await sendTrafficPayload("/state", {
          session_id: activeSessionId,
          state: "red",
          message: "Waiting for user permission / input",
        });
      }
    },

    "tool.execute.after": async (input, output) => {
      if (isSessionActive) {
        const isError = output && output.output && (output.output.includes("error:") || output.output.includes("Error:") || output.output.includes("Failed"));
        await sendTrafficPayload("/state", {
          session_id: activeSessionId,
          state: isError ? "red" : "green",
          message: isError ? "Tool execution error" : "Ready",
        });
      }
    },
  };
};

export default TrafficLightPlugin;
