// generated-by: traffic-status
import { tool } from "@opencode-ai/plugin";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";

const PORT = process.env.TRAFFIC_STATUS_PORT || "8765";
const BASE_URL = `http://127.0.0.1:${PORT}`;

async function isDaemonRunning() {
  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 350);
    const res = await fetch(`${BASE_URL}/ping`, { signal: controller.signal });
    clearTimeout(timeout);
    return res.ok;
  } catch {
    return false;
  }
}

function getCandidateBinPaths() {
  const isWin = os.platform() === "win32";
  const binName = isWin ? "traffic-status.exe" : "traffic-status";
  const candidates = [
    binName,
    isWin
      ? path.join(process.env.LOCALAPPDATA || path.join(os.homedir(), "AppData", "Local"), "traffic-status", "bin", binName)
      : path.join(os.homedir(), ".local", "share", "traffic-status", "bin", binName),
    path.join(os.homedir(), ".cargo", "bin", binName),
  ];
  return candidates;
}

async function ensureDaemonRunning() {
  if (await isDaemonRunning()) return true;

  const candidates = getCandidateBinPaths();
  for (const bin of candidates) {
    try {
      if (bin.includes(path.sep) && !fs.existsSync(bin)) continue;
      const child = spawn(bin, ["daemon"], {
        detached: true,
        stdio: "ignore",
        windowsHide: true,
      });
      child.unref();

      for (let i = 0; i < 20; i++) {
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
    const res = await fetch(`${BASE_URL}${endpoint}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    return res.ok;
  } catch (e) {
    if (!autoStart && (await ensureDaemonRunning())) {
      try {
        const retryRes = await fetch(`${BASE_URL}${endpoint}`, {
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

export const TrafficStatusPlugin = async (_ctx) => {
  return {
    tool: {
      traffic: tool({
        description: "Control floating desktop traffic status indicator (on, off, green, yellow, red)",
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
              ? "Traffic Status activated (Green) 🟢"
              : "Failed to connect to Traffic Status daemon. Ensure traffic-status is installed.";
          }
          if (args.action === "off") {
            isSessionActive = false;
            await sendTrafficPayload("/session/off", {
              session_id: activeSessionId,
            });
            return "Traffic Status dismissed ⚪";
          }
          if (isSessionActive) {
            await sendTrafficPayload("/state", {
              session_id: activeSessionId,
              state: args.action,
              message: args.message || "",
            });
          }
          return `Traffic Status state set to ${args.action}`;
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

export default TrafficStatusPlugin;
