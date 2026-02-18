SUMMARY = "Node.js (native) - pre-built binaries"
DESCRIPTION = "Node.js is a JavaScript runtime built on Chrome's V8 JavaScript engine. \
This recipe downloads pre-built binaries for the build host instead of compiling from source."
HOMEPAGE = "https://nodejs.org/"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://LICENSE;md5=d49f2b1599f25f1d420fc6717a4067b2"

DEPENDS = ""
PROVIDES = "nodejs-native"

inherit native

NODEJS_VERSION = "24.13.1"

# Determine the correct architecture for the build host
def get_nodejs_arch(d):
    import platform
    build_arch = d.getVar('BUILD_ARCH')
    build_os = d.getVar('BUILD_OS')
    
    # Map BUILD_ARCH to Node.js architecture naming
    arch_map = {
        'x86_64': 'x64',
        'aarch64': 'arm64',
        'arm64': 'arm64',
        'i686': 'x86',
        'i586': 'x86',
    }
    
    nodejs_arch = arch_map.get(build_arch, build_arch)
    
    # Determine OS
    if 'linux' in build_os.lower():
        return f'linux-{nodejs_arch}'
    else:
        bb.fatal(f"Unsupported build OS: {build_os}")

NODEJS_ARCH = "${@get_nodejs_arch(d)}"

# Checksums for different architectures
def get_nodejs_checksum(d):
    arch = d.getVar('NODEJS_ARCH')

    checksums = {
        'linux-x64': '30215f90ea3cd04dfbc06e762c021393fa173a1d392974298bbc871a8e461089',
        'linux-arm64': 'c827d3d301e2eed1a51f36d0116b71b9e3d9e3b728f081615270ea40faac34c1',
    }
    
    checksum = checksums.get(arch)
    if not checksum:
        bb.fatal(f"No checksum available for architecture: {arch}")
    return checksum

SRC_URI = "https://nodejs.org/dist/v${NODEJS_VERSION}/node-v${NODEJS_VERSION}-${NODEJS_ARCH}.tar.xz"
SRC_URI[sha256sum] = "${@get_nodejs_checksum(d)}"

S = "${UNPACKDIR}/node-v${NODEJS_VERSION}-${NODEJS_ARCH}"

# Network access needed to download
do_fetch[network] = "1"

# No compilation needed, just extraction
do_configure[noexec] = "1"
do_compile[noexec] = "1"

do_install() {
    # Install everything to native sysroot
    install -d ${D}${prefix}
    
    # Copy all files from the extracted archive
    cp -r ${S}/bin ${D}${prefix}/
    cp -r ${S}/lib ${D}${prefix}/
    cp -r ${S}/include ${D}${prefix}/
    cp -r ${S}/share ${D}${prefix}/
    
    # Copy license file
    install -d ${D}${datadir}/licenses/${PN}
    cp ${S}/LICENSE ${D}${datadir}/licenses/${PN}/
}

do_install:append() {
    # Create symlinks for compatibility
    ln -sf node ${D}${bindir}/nodejs
    
    # Ensure npm and npx are executable
    chmod +x ${D}${prefix}/lib/node_modules/npm/bin/npm-cli.js
    chmod +x ${D}${prefix}/lib/node_modules/npm/bin/npx-cli.js
}

FILES:${PN} = " \
    ${bindir}/* \
    ${libdir}/* \
    ${includedir}/* \
    ${datadir}/* \
    ${prefix}/lib/node_modules/* \
"

INSANE_SKIP:${PN} += "already-stripped file-rdeps"

# Prevent QA warnings for pre-built binaries
INSANE_SKIP:${PN} += "ldflags libdir staticdev"
