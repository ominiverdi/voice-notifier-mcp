#!/usr/bin/env bash
set -euo pipefail

readonly MODEL_URL="https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX/resolve/main/onnx/model.onnx"
readonly VOICE_URL="https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX/resolve/main/voices/bf_emma.bin"
readonly MODEL_SHA256="8fbea51ea711f2af382e88c833d9e288c6dc82ce5e98421ea61c058ce21a34cb"
readonly VOICE_SHA256="669fe0647f9dd04fcab92f1439a40eeb4c8b4ab1f82e4996fe3d918ce4a63b73"
readonly DATA_HOME="${XDG_DATA_HOME:-${HOME}/.local/share}"
readonly DESTINATION="${VOICE_NOTIFIER_ASSET_DIR:-${DATA_HOME}/voice-notifier-mcp}"

for command in curl sha256sum mktemp; do
    if ! command -v "${command}" >/dev/null 2>&1; then
        printf 'Required command not found: %s\n' "${command}" >&2
        exit 1
    fi
done

mkdir -p "${DESTINATION}"
model_temp="$(mktemp "${DESTINATION}/.model.onnx.XXXXXX")"
voice_temp="$(mktemp "${DESTINATION}/.bf_emma.bin.XXXXXX")"
cleanup() {
    rm -f "${model_temp}" "${voice_temp}"
}
trap cleanup EXIT

printf 'Downloading Kokoro model to %s\n' "${DESTINATION}"
curl --fail --location --retry 3 --output "${model_temp}" "${MODEL_URL}"
printf '%s  %s\n' "${MODEL_SHA256}" "${model_temp}" | sha256sum --check --status

printf 'Downloading bf_emma voice\n'
curl --fail --location --retry 3 --output "${voice_temp}" "${VOICE_URL}"
printf '%s  %s\n' "${VOICE_SHA256}" "${voice_temp}" | sha256sum --check --status

chmod 0644 "${model_temp}" "${voice_temp}"
mv "${model_temp}" "${DESTINATION}/model.onnx"
mv "${voice_temp}" "${DESTINATION}/bf_emma.bin"
trap - EXIT

printf 'Installed and verified Kokoro assets in %s\n' "${DESTINATION}"
