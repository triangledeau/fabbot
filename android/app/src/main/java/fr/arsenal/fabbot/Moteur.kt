package fr.arsenal.fabbot

/**
 * Le lien avec la bibliothèque Rust `libfab_android.so` (crates/android,
 * fichier src/jni_lien.rs).
 */
object Moteur {
    init {
        System.loadLibrary("fab_android")
    }

    /**
     * Démarre le serveur d'Arsenal dans le téléphone, avec [dossier] comme
     * dossier de travail (decks, sauvegardes), et rend son port, ou -1 en
     * cas d'erreur (détail dans le journal : `adb logcat -s fab-android`).
     * Un second appel dans le même processus rend le même port.
     */
    @JvmStatic
    external fun demarrer(dossier: String): Int
}
