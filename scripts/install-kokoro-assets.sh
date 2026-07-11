#!/usr/bin/env bash
set -euo pipefail

readonly MODEL_URL="https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX/resolve/main/onnx/model.onnx"
readonly VOICE_BASE_URL="https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX/resolve/main/voices"
readonly MODEL_SHA256="8fbea51ea711f2af382e88c833d9e288c6dc82ce5e98421ea61c058ce21a34cb"
readonly -a VOICE_NAMES=(
    af_bella
    af_heart
    af_nicole
    af_sarah
    am_michael
    bf_emma
)
declare -Ar VOICE_SHA256=(
    [af_bella]="f69d836209b78eb8c66e75e3cda491e26ea838a3674257e9d4e5703cbaf55c8b"
    [af_heart]="d583ccff3cdca2f7fae535cb998ac07e9fcb90f09737b9a41fa2734ec44a8f0b"
    [af_nicole]="cd2191ab31b914ed7b318416b0e4440fdf392ddad9106a060819aa600a64f59a"
    [af_sarah]="4409fbc125afabacc615d94db5398d847006a737b0247d6892b7a9a0007a2f0a"
    [am_michael]="1d1f21dd8da39c30705cd4c75d039d265e9bc4a2a93ed09bc9e1b1225eb95ba1"
    [bf_emma]="669fe0647f9dd04fcab92f1439a40eeb4c8b4ab1f82e4996fe3d918ce4a63b73"
)
readonly DATA_HOME="${XDG_DATA_HOME:-${HOME}/.local/share}"
readonly DESTINATION="${VOICE_NOTIFIER_ASSET_DIR:-${DATA_HOME}/voice-notifier-mcp}"

for command in curl sha256sum mktemp; do
    if ! command -v "${command}" >/dev/null 2>&1; then
        printf 'Required command not found: %s\n' "${command}" >&2
        exit 1
    fi
done

mkdir -p "${DESTINATION}"
declare -a temporary_files=()
cleanup() {
    rm -f "${temporary_files[@]}"
}
trap cleanup EXIT

download_verified() {
    local url="$1"
    local expected_sha256="$2"
    local destination="$3"
    local label="$4"

    if [[ -f "${destination}" ]] &&
        printf '%s  %s\n' "${expected_sha256}" "${destination}" |
            sha256sum --check --status; then
        printf 'Already installed and verified: %s\n' "${label}"
        return
    fi

    local temporary
    temporary="$(mktemp "${DESTINATION}/.${label}.XXXXXX")"
    temporary_files+=("${temporary}")
    printf 'Downloading %s\n' "${label}"
    curl --fail --location --retry 3 --output "${temporary}" "${url}"
    printf '%s  %s\n' "${expected_sha256}" "${temporary}" |
        sha256sum --check --status
    chmod 0644 "${temporary}"
    mv "${temporary}" "${destination}"
}

download_verified \
    "${MODEL_URL}" \
    "${MODEL_SHA256}" \
    "${DESTINATION}/model.onnx" \
    "model.onnx"

for voice_name in "${VOICE_NAMES[@]}"; do
    download_verified \
        "${VOICE_BASE_URL}/${voice_name}.bin" \
        "${VOICE_SHA256[${voice_name}]}" \
        "${DESTINATION}/${voice_name}.bin" \
        "${voice_name}.bin"
done

trap - EXIT
printf 'Installed and verified Kokoro assets in %s\n' "${DESTINATION}"
