# meta-nodejs-native-bin

A Yocto/OpenEmbedded layer that provides pre-built Node.js binaries for the build host, significantly reducing build times compared to building nodejs-native from source.

## Why Use This Layer?

Building `nodejs-native` from source can take 30+ minutes depending on your build host. This layer downloads pre-built Node.js binaries from the official Node.js distribution, reducing the install time to seconds.

## Supported Platforms

- Linux x86_64 (x64)
- Linux ARM64 (aarch64)
- macOS x86_64
- macOS ARM64 (Apple Silicon)

## Installation

1. Add this layer to your build environment:

    ```bash
    cd /path/to/your/build
    bitbake-layers add-layer /path/to/meta-nodejs-native-bin
    ```

2. Make sure the layer has higher priority than layers providing `nodejs-native` from source. The default priority is set to 10 in `layer.conf`.

## Usage

Simply list `nodejs-native` as a dependency in your recipe, and this layer will provide it:

```bitbake
DEPENDS += "nodejs-native"
```

The layer provides the same functionality as the standard `nodejs-native` recipe but uses pre-built binaries instead of compiling from source.

## Node.js Version

Current version: **22.13.1** (LTS)

To update to a different version, edit `recipes-devtools/nodejs/nodejs-native_VERSION.bb` and update:

- The filename to match the new version
- `NODEJS_VERSION` variable
- SHA256 checksums for each supported architecture

You can find checksums on the official Node.js download page:
https://nodejs.org/dist/

## Layer Dependencies

- openembedded-core (core)

## Layer Compatibility

Compatible with Yocto releases:

- Kirkstone (4.0)
- Langdale (4.1)
- Mickledore (4.2)
- Nanbield (4.3)
- Scarthgap (5.0)
- Styhead (5.1+)

## Testing

To verify the layer works correctly, build a recipe that depends on `nodejs-native`:

```bash
bitbake -c cleansstate nodejs-native
bitbake nodejs-native
```

The build should complete in seconds rather than minutes.

## License

MIT License - See LICENSE file for details

## Contributing

Contributions are welcome! Please submit pull requests or issues on the project repository.
