// L'appli Kotlin : une seule activité, une WebView qui affiche la page
// d'Arsenal servie par la bibliothèque Rust `libfab_android.so`
// (crates/android). Les bibliothèques Rust sont rangées dans
// src/main/jniLibs/ par android/construire.sh, avant Gradle.
//
// Signature : si les variables d'environnement ARSENAL_CLE (chemin du
// fichier .jks), ARSENAL_CLE_MDP (mot de passe) et, au besoin,
// ARSENAL_CLE_ALIAS (nom de la clé, « arsenal » par défaut) existent,
// l'APK « release » est signé avec cette clé. Sinon, il est signé avec la
// clé de débogage de la machine, et Gradle le dit : un tel APK ne pourra
// pas être mis à jour par un APK signé avec la vraie clé sans
// désinstaller (ce qui efface les sauvegardes). Voir docs/android.md, 8.3.

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

val cle: String? = System.getenv("ARSENAL_CLE")?.takeIf { it.isNotBlank() }
val cleMdp: String? = System.getenv("ARSENAL_CLE_MDP")?.takeIf { it.isNotBlank() }
val cleAlias: String = System.getenv("ARSENAL_CLE_ALIAS")?.takeIf { it.isNotBlank() } ?: "arsenal"
val avecVraieCle = cle != null && cleMdp != null && file(cle).isFile

// Numéro de version : donné par GitHub (numéro de la construction), sinon 1.
// Android refuse une mise à jour dont le numéro est plus petit.
val numeroVersion: Int = System.getenv("ARSENAL_VERSION")?.toIntOrNull() ?: 1

android {
    namespace = "fr.arsenal.fabbot"
    compileSdk = 35

    defaultConfig {
        applicationId = "fr.arsenal.fabbot"
        minSdk = 26
        targetSdk = 35
        versionCode = numeroVersion
        versionName = "0.2.$numeroVersion"
    }

    signingConfigs {
        if (avecVraieCle) {
            create("arsenal") {
                storeFile = file(cle!!)
                storePassword = cleMdp
                keyAlias = cleAlias
                keyPassword = cleMdp
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            signingConfig = if (avecVraieCle) {
                signingConfigs.getByName("arsenal")
            } else {
                logger.warn(
                    "ATTENTION : ARSENAL_CLE ou ARSENAL_CLE_MDP absent. " +
                        "APK signé avec la clé de débogage (voir docs/android.md, 8.3)."
                )
                signingConfigs.getByName("debug")
            }
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }

    // Le fichier des textes des cartes (33 Mo) est déjà du texte : le
    // laisser non compressé accélère sa copie au premier lancement.
    androidResources {
        noCompress += "csv"
    }
}

dependencies {
    implementation("androidx.appcompat:appcompat:1.7.0")
}
