#!/usr/bin/env bash
# Construit l'APK d'Arsenal (docs/android.md, section 8.2).
#
#   bash android/construire.sh
#   → android/app/build/outputs/apk/release/app-release.apk
#
# Il faut : Rust, cargo-ndk (`cargo install cargo-ndk`), le SDK Android
# (ANDROID_HOME) avec un NDK (ANDROID_NDK_HOME), et un JDK 17 ou plus.
#
# Réglages, par variables d'environnement :
#   CIBLES            processeurs visés, « arm64-v8a x86_64 » par défaut
#                     (arm64-v8a : le téléphone ; x86_64 : l'émulateur) ;
#   ARSENAL_CLE, ARSENAL_CLE_MDP, ARSENAL_CLE_ALIAS
#                     la clé de signature (voir android/app/build.gradle.kts) ;
#   ARSENAL_VERSION   numéro de version de l'APK (1 par défaut).
#
# Les textes officiels des cartes (data/cards/sources/public_card_data.csv,
# hors dépôt) sont téléchargés s'ils manquent, puis rangés dans l'APK.
# Le vrai dos de carte est embarqué s'il est dans data/outils/ (voir
# crates/android/build.rs).
set -euo pipefail
cd "$(dirname "$0")/.."

CIBLES=${CIBLES:-"arm64-v8a x86_64"}
args=()
for c in $CIBLES; do
    case "$c" in
        arm64-v8a) triplet=aarch64-linux-android ;;
        x86_64) triplet=x86_64-linux-android ;;
        armeabi-v7a) triplet=armv7-linux-androideabi ;;
        *) echo "Processeur inconnu : $c" >&2; exit 1 ;;
    esac
    if command -v rustup >/dev/null; then rustup target add "$triplet" >/dev/null; fi
    args+=(-t "$c")
done

echo "== 1. Bibliothèque Rust ($CIBLES)"
# Pages mémoire de 16 Ko (obligatoires sur certains téléphones récents)
# et bibliothèque sans symboles de débogage (plus petite).
export CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=16384"
export CARGO_TARGET_X86_64_LINUX_ANDROID_RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=16384"
export CARGO_PROFILE_RELEASE_STRIP=symbols
rm -rf android/app/src/main/jniLibs
cargo ndk "${args[@]}" -o android/app/src/main/jniLibs build --release -p fab-android

echo "== 2. Textes officiels des cartes"
csv=data/cards/sources/public_card_data.csv
if [ ! -f "$csv" ]; then
    bash tools/fetch_cards.sh || echo "Téléchargement impossible : l'APK sera fait sans les textes des cartes."
fi
assets=android/app/src/main/assets/donnees
rm -rf "$assets"
if [ -f "$csv" ]; then
    mkdir -p "$assets"
    cp "$csv" "$assets/"
    echo "Textes rangés dans l'APK."
fi

echo "== 3. APK"
cd android
./gradlew --no-daemon assembleRelease
echo "APK prêt : android/app/build/outputs/apk/release/app-release.apk"
