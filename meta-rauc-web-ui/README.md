# meta-rauc-web-ui

This is a Yocto/OpenEmbedded layer providing the RAUC Web UI application.

## Description

This layer provides recipes for building and deploying the RAUC Web UI, a web-based interface for managing RAUC update bundles on embedded Linux systems.

## Dependencies

This layer depends on:

- openembedded-core (meta)

* meta-rust-bin (for cargo support)
* meta-oe (for nodejs-native from meta-openembedded)

## Adding the Layer

To add this layer to your Yocto build, add it to your `bblayers.conf`:

```bitbake
BBLAYERS += "/path/to/rauc-web-ui/meta-rauc-web-ui"
```

Or if using kas, add to your kas configuration:

```yaml
repos:
    rauc-web-ui:
        url: https://github.com/hrueger/rauc-web-ui.git
        layers:
            meta-rauc-web-ui:
```

## Build Process

The recipe builds both the Svelte UI and the Rust backend:

1. **UI Build (on host)**: The Svelte frontend is built using pnpm on the build host before Rust compilation
2. **Rust Build (cross-compiled)**: The Rust backend is cross-compiled for the target architecture

The `SKIP_UI_BUILD` environment variable is set during the Yocto build to prevent the Rust build script from attempting to build the UI again (which would fail due to pnpm not being available in the target environment).

## Components Installed

The recipe installs:

- `/usr/bin/rauc-web-ui` - Main application binary
- `/etc/rauc-web-ui/config.env` - Configuration file
- `/var/lib/rauc-web-ui/tmp` - Upload temporary directory
- systemd service unit (enabled by default)

## Configuration

Edit `/etc/rauc-web-ui/config.env` on the target to configure:

- `UPLOAD_TMP_DIR` - Temporary directory for bundle uploads
- Optional SSH configuration for remote development

## Usage

After building and deploying to your image:

```bash
bitbake rauc-web-ui
```

The service will start automatically and listen on port 8000 by default.

## Compatibility

- Layer series: whinlatter
- Tested with Yocto 5.1 (whinlatter)

## Maintainer

This layer is maintained as part of the RAUC Web UI project.
