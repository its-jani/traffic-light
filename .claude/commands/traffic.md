<!-- generated-by: traffic-status -->
# Traffic Status Slash Command (/traffic)

Execute the corresponding action immediately in 1 step without reading source files or repository searches.

### Control & Auto-Start:

- **`/traffic on`** (or `/traffic`):
  Starts the floating status daemon (if not already running) and registers the active session.
  ```powershell
  traffic-status on "Claude"
  ```

- **`/traffic yellow [msg]`** (or when processing/thinking/running tools):
  ```powershell
  traffic-status yellow "$ARG"
  ```

- **`/traffic green [msg]`** (or when ready/waiting for prompt):
  ```powershell
  traffic-status green "$ARG"
  ```

- **`/traffic red [msg]`** (or when error/needs confirmation):
  ```powershell
  traffic-status red "$ARG"
  ```

- **`/traffic off`**:
  ```powershell
  traffic-status off
  ```
