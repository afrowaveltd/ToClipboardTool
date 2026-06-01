#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

usage() {
    echo "Usage: $0 <binary> <version> <target> <output-dir>"
    echo ""
    echo "Example:"
    echo "  $0 target/x86_64-unknown-linux-gnu/release/to-clipboard 0.1.0 x86_64-unknown-linux-gnu dist"
}

if [[ $# -ne 4 ]]; then
    usage
    exit 64
fi

BINARY_PATH="$1"
VERSION="${2#v}"
TARGET="$3"
OUTPUT_DIR="$4"

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACKAGE_NAME="to-clipboard"
DESCRIPTION="Copy command stdout or piped stdin to the OS clipboard, and pipe clipboard text into commands."

if [[ ! -f "$BINARY_PATH" ]]; then
    echo "error: binary not found: $BINARY_PATH" >&2
    exit 1
fi

case "$TARGET" in
    x86_64-unknown-linux-gnu)
        DEB_ARCH="amd64"
        RPM_ARCH="x86_64"
        ARCH_LABEL="linux-x86_64"
        ;;
    aarch64-unknown-linux-gnu)
        DEB_ARCH="arm64"
        RPM_ARCH="aarch64"
        ARCH_LABEL="linux-arm64"
        ;;
    *)
        echo "error: unsupported Linux target: $TARGET" >&2
        exit 64
        ;;
esac

mkdir -p "$OUTPUT_DIR"
OUTPUT_DIR="$(cd "$OUTPUT_DIR" && pwd)"
WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

copy_docs() {
    local doc_dir="$1"

    mkdir -p "$doc_dir/assets"
    cp "$PROJECT_DIR/README.md" "$doc_dir/README.md"
    cp "$PROJECT_DIR/LICENSE" "$doc_dir/LICENSE"
    cp "$PROJECT_DIR/Docs/en.md" "$doc_dir/DOCUMENTATION.md"
    cp "$PROJECT_DIR/assets/app-icon.svg" "$doc_dir/assets/app-icon.svg"
    chmod 0644 "$doc_dir/README.md" "$doc_dir/LICENSE" "$doc_dir/DOCUMENTATION.md" "$doc_dir/assets/app-icon.svg"
}

make_archive() {
    local root="$WORK_DIR/${PACKAGE_NAME}-${VERSION}-${ARCH_LABEL}"
    local archive="$OUTPUT_DIR/${PACKAGE_NAME}-${VERSION}-${ARCH_LABEL}.tar.gz"

    mkdir -p "$root/bin"
    cp "$BINARY_PATH" "$root/bin/$PACKAGE_NAME"
    copy_docs "$root/share/doc/$PACKAGE_NAME"

    tar -C "$WORK_DIR" -czf "$archive" "$(basename "$root")"
    echo "$archive"
}

make_deb() {
    local root="$WORK_DIR/deb"
    local deb="$OUTPUT_DIR/${PACKAGE_NAME}_${VERSION}_${DEB_ARCH}.deb"

    mkdir -p "$root/DEBIAN" "$root/usr/bin" "$root/usr/share/doc/$PACKAGE_NAME"
    cp "$BINARY_PATH" "$root/usr/bin/$PACKAGE_NAME"
    copy_docs "$root/usr/share/doc/$PACKAGE_NAME"
    chmod 0755 "$root/usr/bin/$PACKAGE_NAME"

    cat > "$root/DEBIAN/control" <<EOF
Package: $PACKAGE_NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: $DEB_ARCH
Maintainer: Afrowave <support@afrowave.com>
Homepage: https://github.com/afrowaveltd/ToClipboardTool
Description: $DESCRIPTION
EOF

    dpkg-deb --build --root-owner-group "$root" "$deb" >/dev/null
    echo "$deb"
}

make_rpm() {
    local rpmbuild_root="$WORK_DIR/rpmbuild"
    local sources="$rpmbuild_root/SOURCES"
    local spec="$rpmbuild_root/SPECS/$PACKAGE_NAME.spec"

    mkdir -p "$rpmbuild_root/BUILD" "$rpmbuild_root/BUILDROOT" "$rpmbuild_root/RPMS" "$rpmbuild_root/SOURCES" "$rpmbuild_root/SPECS" "$rpmbuild_root/SRPMS" "$rpmbuild_root/TMP"
    mkdir -p "$sources/usr/bin" "$sources/usr/share/doc/$PACKAGE_NAME/assets" "$sources/usr/share/licenses/$PACKAGE_NAME"
    cp "$BINARY_PATH" "$sources/usr/bin/$PACKAGE_NAME"
    cp "$PROJECT_DIR/README.md" "$sources/usr/share/doc/$PACKAGE_NAME/README.md"
    cp "$PROJECT_DIR/Docs/en.md" "$sources/usr/share/doc/$PACKAGE_NAME/DOCUMENTATION.md"
    cp "$PROJECT_DIR/assets/app-icon.svg" "$sources/usr/share/doc/$PACKAGE_NAME/assets/app-icon.svg"
    cp "$PROJECT_DIR/LICENSE" "$sources/usr/share/licenses/$PACKAGE_NAME/LICENSE"
    chmod 0755 "$sources/usr/bin/$PACKAGE_NAME"
    chmod 0644 "$sources/usr/share/doc/$PACKAGE_NAME/README.md" "$sources/usr/share/doc/$PACKAGE_NAME/DOCUMENTATION.md" "$sources/usr/share/doc/$PACKAGE_NAME/assets/app-icon.svg" "$sources/usr/share/licenses/$PACKAGE_NAME/LICENSE"

    cat > "$spec" <<EOF
Name: $PACKAGE_NAME
Version: $VERSION
Release: 1%{?dist}
Summary: $DESCRIPTION
License: MIT
URL: https://github.com/afrowaveltd/ToClipboardTool

%description
$DESCRIPTION

%prep

%build

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/doc/$PACKAGE_NAME/assets
mkdir -p %{buildroot}/usr/share/licenses/$PACKAGE_NAME
cp %{_sourcedir}/usr/bin/$PACKAGE_NAME %{buildroot}/usr/bin/$PACKAGE_NAME
cp %{_sourcedir}/usr/share/doc/$PACKAGE_NAME/README.md %{buildroot}/usr/share/doc/$PACKAGE_NAME/README.md
cp %{_sourcedir}/usr/share/doc/$PACKAGE_NAME/DOCUMENTATION.md %{buildroot}/usr/share/doc/$PACKAGE_NAME/DOCUMENTATION.md
cp %{_sourcedir}/usr/share/doc/$PACKAGE_NAME/assets/app-icon.svg %{buildroot}/usr/share/doc/$PACKAGE_NAME/assets/app-icon.svg
cp %{_sourcedir}/usr/share/licenses/$PACKAGE_NAME/LICENSE %{buildroot}/usr/share/licenses/$PACKAGE_NAME/LICENSE

%files
/usr/bin/$PACKAGE_NAME
%doc /usr/share/doc/$PACKAGE_NAME/README.md
%doc /usr/share/doc/$PACKAGE_NAME/DOCUMENTATION.md
%doc /usr/share/doc/$PACKAGE_NAME/assets/app-icon.svg
%license /usr/share/licenses/$PACKAGE_NAME/LICENSE
EOF

    rpmbuild --quiet \
        --define "_topdir $rpmbuild_root" \
        --define "_tmppath $rpmbuild_root/TMP" \
        --target "$RPM_ARCH" \
        -bb "$spec"

    local rpm
    rpm="$(find "$rpmbuild_root/RPMS" -type f -name "*.rpm" -print -quit)"
    cp "$rpm" "$OUTPUT_DIR/${PACKAGE_NAME}-${VERSION}-1.${RPM_ARCH}.rpm"
    echo "$OUTPUT_DIR/${PACKAGE_NAME}-${VERSION}-1.${RPM_ARCH}.rpm"
}

make_archive
make_deb
make_rpm
