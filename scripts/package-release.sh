#!/usr/bin/env bash
set -euo pipefail

readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${PROJECT_ROOT}"

for command in cargo dpkg-deb find install sha256sum strip tar touch; do
    if ! command -v "${command}" >/dev/null 2>&1; then
        printf 'Required command not found: %s\n' "${command}" >&2
        exit 1
    fi
done

if ! cargo generate-rpm --version >/dev/null 2>&1; then
    printf 'Required Cargo subcommand not found: cargo-generate-rpm\n' >&2
    exit 1
fi

readonly VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
readonly SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct 2>/dev/null || date +%s)}"
export SOURCE_DATE_EPOCH
readonly MACHINE_ARCH="$(uname -m)"
case "${MACHINE_ARCH}" in
    x86_64)
        readonly DEB_ARCH="amd64"
        readonly RPM_ARCH="x86_64"
        ;;
    aarch64)
        readonly DEB_ARCH="arm64"
        readonly RPM_ARCH="aarch64"
        ;;
    *)
        printf 'Unsupported packaging architecture: %s\n' "${MACHINE_ARCH}" >&2
        exit 1
        ;;
esac

readonly DIST_DIR="${PROJECT_ROOT}/dist"
readonly WORK_DIR="${PROJECT_ROOT}/target/package-release"
readonly BINARY="${PROJECT_ROOT}/target/release/voice-notifier-mcp"
readonly ASSET_INSTALLER="${PROJECT_ROOT}/scripts/install-kokoro-assets.sh"

rm -rf "${DIST_DIR}" "${WORK_DIR}"
mkdir -p "${DIST_DIR}" "${WORK_DIR}"

cargo build --locked --release
strip --strip-unneeded "${BINARY}"

archive_name="voice-notifier-mcp-v${VERSION}-linux-${MACHINE_ARCH}"
archive_root="${WORK_DIR}/${archive_name}"
install -Dm0755 "${BINARY}" "${archive_root}/voice-notifier-mcp"
install -Dm0755 "${ASSET_INSTALLER}" "${archive_root}/voice-notifier-install-assets"
install -Dm0644 README.md "${archive_root}/README.md"
install -Dm0644 LICENSE "${archive_root}/LICENSE"
install -Dm0644 THIRD_PARTY.md "${archive_root}/THIRD_PARTY.md"
tar \
    --sort=name \
    --mtime="@${SOURCE_DATE_EPOCH}" \
    --owner=0 \
    --group=0 \
    --numeric-owner \
    -C "${WORK_DIR}" \
    -czf "${DIST_DIR}/${archive_name}.tar.gz" \
    "${archive_name}"

deb_root="${WORK_DIR}/deb"
install -Dm0755 "${BINARY}" "${deb_root}/usr/bin/voice-notifier-mcp"
install -Dm0755 "${ASSET_INSTALLER}" "${deb_root}/usr/bin/voice-notifier-install-assets"
install -Dm0644 README.md "${deb_root}/usr/share/doc/voice-notifier-mcp/README.md"
install -Dm0644 CHANGELOG.md "${deb_root}/usr/share/doc/voice-notifier-mcp/changelog"
install -Dm0644 LICENSE "${deb_root}/usr/share/doc/voice-notifier-mcp/copyright"
install -Dm0644 THIRD_PARTY.md "${deb_root}/usr/share/doc/voice-notifier-mcp/THIRD_PARTY.md"
installed_size="$(du -sk "${deb_root}/usr" | cut -f1)"
mkdir -p "${deb_root}/DEBIAN"
cat >"${deb_root}/DEBIAN/control" <<EOF
Package: voice-notifier-mcp
Version: ${VERSION}-1
Section: sound
Priority: optional
Architecture: ${DEB_ARCH}
Maintainer: Lorenzo Becchi <4244619+ominiverdi@users.noreply.github.com>
Installed-Size: ${installed_size}
Depends: libc6 (>= 2.39), libstdc++6, libgcc-s1, pipewire-bin, libnotify-bin, curl, coreutils, bash
Recommends: speech-dispatcher
Homepage: https://github.com/ominiverdi/voice-notifier-mcp
Description: Local MCP notification server with neural voice
 Voice Notifier MCP provides desktop, terminal, and local Kokoro speech
 notifications to Model Context Protocol clients.
EOF
find "${deb_root}" -exec touch --date="@${SOURCE_DATE_EPOCH}" {} +

deb_name="voice-notifier-mcp_${VERSION}-1_${DEB_ARCH}.deb"
dpkg-deb --root-owner-group --build "${deb_root}" "${DIST_DIR}/${deb_name}"

cargo generate-rpm \
    --arch "${RPM_ARCH}" \
    --auto-req disabled \
    --source-date "${SOURCE_DATE_EPOCH}" \
    --output "${DIST_DIR}"

(
    cd "${DIST_DIR}"
    sha256sum ./*.deb ./*.rpm ./*.tar.gz >SHA256SUMS
)

printf 'Release packages created in %s\n' "${DIST_DIR}"
