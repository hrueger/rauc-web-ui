SUMMARY = "RAUC Web UI"
DESCRIPTION = "Upload and manage RAUC update bundles through a web interface"
HOMEPAGE = "https://github.com/hrueger/rauc-web-ui"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

inherit cargo_bin systemd

# Build the UI on the build host, not the target
DEPENDS += "nodejs-native"

# Enable network for the compile task allowing cargo to download dependencies
do_compile[network] = "1"

SRC_URI = "git://github.com/hrueger/rauc-web-ui.git;branch=main;protocol=https \
           file://rauc-web-ui.service \
           file://config.env \
          "

SRCREV = "${AUTOREV}"

# Set environment variable to skip UI build in build.rs
# since we're building it separately on the host
export SKIP_UI_BUILD = "1"

# Build the UI before the Rust compilation
do_compile:prepend() {
    cd ${S}/ui
    bbnote "Building Svelte UI with pnpm on build host..."
    
    # Install pnpm globally using npm
    npm install -g pnpm
    
    # Build the UI
    pnpm install
    pnpm build
    cd ${S}
}

do_install:append() {
    # Install systemd service
    install -d ${D}${systemd_system_unitdir}
    install -m 0644 ${UNPACKDIR}/rauc-web-ui.service ${D}${systemd_system_unitdir}/

    # Install configuration
    install -d ${D}${sysconfdir}/rauc-web-ui
    install -m 0644 ${UNPACKDIR}/config.env ${D}${sysconfdir}/rauc-web-ui/

    # Create upload directory
    install -d ${D}/var/lib/rauc-web-ui/tmp
}

SYSTEMD_SERVICE:${PN} = "rauc-web-ui.service"
SYSTEMD_AUTO_ENABLE = "enable"

FILES:${PN} += " \
    ${systemd_system_unitdir}/rauc-web-ui.service \
    ${sysconfdir}/rauc-web-ui/config.env \
    /var/lib/rauc-web-ui/tmp \
"
