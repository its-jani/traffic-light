import { tool } from "@opencode-ai/plugin";

async function sendTrafficPayload(endpoint, body) {
  try {
    const res = await fetch(`http://127.0.0.1:8765${endpoint}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    return res.ok;
  } catch (e) {
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
            await sendTrafficPayload("/session/on", {
              session_id: activeSessionId,
              label: "OpenCode",
              state: "green",
            });
            return "Traffic Light activated (Green) 🟢";
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
