# RAUC Web UI

Modern web interface for RAUC (Robust Auto-Update Controller) built with Rust, Rocket, and Svelte.

## Development Setup

### Prerequisites

- Rust and Cargo
- Node.js and pnpm (for UI development)
- `sshpass` (for development mode with SSH)

### Configuration

1. Copy `.env.example` to `.env`:

    ```bash
    cp .env.example .env
    ```

2. Edit `.env` with your SSH credentials:
    ```env
    SSH_HOST=root@172.16.220.172
    SSH_PASSWORD=your_actual_password
    UPLOAD_TMP_DIR=/data/tmp
    ```

### Building

The UI is automatically built as part of the Rust build process via `build.rs`:

```bash
cargo build
```

This will:

1. Run `pnpm build` in the `ui/` directory
2. Generate static files to `static/` directory
3. Build the Rust application

### Running in Development Mode

1. Run:

    ```bash
    cargo run
    ```

1. Open http://localhost:8000 in your browser

## Yocto/OpenEmbedded Integration

This repository includes a Yocto layer for building RAUC Web UI in embedded Linux images.

### Adding the Layer

Add the `meta-rauc-web-ui` layer to your Yocto build configuration:

**Using bblayers.conf:**

```bitbake
BBLAYERS += "/path/to/rauc-web-ui/meta-rauc-web-ui"
```

**Using kas:**

```yaml
repos:
    rauc-web-ui:
        url: https://github.com/hrueger/rauc-web-ui.git
        layers:
            meta-rauc-web-ui:
```

### Building for Yocto

The Yocto recipe builds the Svelte UI on the build host (not the target), avoiding pnpm/Node.js dependencies on the embedded device. The `SKIP_UI_BUILD` environment variable is automatically set during Yocto builds to prevent the Rust build script from attempting to rebuild the UI.

See [meta-rauc-web-ui/README.md](meta-rauc-web-ui/README.md) for more details.

## Configuration Variables

- `SSH_HOST`: SSH host for remote development (e.g., `root@172.16.220.172`) - Optional
- `SSH_PASSWORD`: SSH password for remote development - Optional (must be set if SSH_HOST is set)
- `UPLOAD_TMP_DIR`: Directory for uploaded bundles (defaults to `/data/tmp`) - Optional

### Theming

- `WEB_UI_TITLE`: Title displayed in the web UI header and title
- `WEB_UI_PRIMARY_COLOR`: Primary accent color
- `WEB_UI_BACKGROUND_COLOR`: Background color
- `WEB_UI_FOREGROUND_COLOR`: Foreground/text color

## API Endpoints for external integration

### GET `/`

Returns a welcome message.

### GET `/api/status`

Returns the current RAUC system status as JSON.

**Example response:**

```json
{
  "compatible": "raspberrypi5",
  "variant": "",
  "booted": "B",
  "boot_primary": "rootfs.1",
  "slots": [...]
}
```

### POST `/api/upload`

Upload an update bundle file. The file is saved to the configured upload directory. In development mode, the bundle is automatically copied to the SSH target via SCP.

**Usage:**

```bash
curl -F "file=@mybundle.raucb" http://localhost:8000/api/upload
```

### GET `/api/bundle-info`

Get information about the uploaded bundle.

**Example response:**

```json
{
  "compatible": "raspberrypi5",
  "version": "v20200703",
  "description": "RAUC Demo Bundle",
  "build": "20260216102744",
  "format": "verity",
  "hash": "50a7b21f54fc59cb5804aeaad30358f32a21b490f5e9c98f8aceea32d78b8211",
  "images": [...]
}
```

### GET `/api/install`

Install the uploaded bundle with streaming progress updates.

**Returns:** Server-Sent Events stream with installation progress

**Example usage:**

```javascript
const response = await fetch("/api/install");
const reader = response.body.getReader();
const decoder = new TextDecoder();

while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    const text = decoder.decode(value);
    console.log(text);
}
```

### POST `/api/reboot`

Reboot the system after installation.

**Returns:** Text confirmation message
