//! Le lien avec Kotlin (seulement sur Android).
//!
//! Côté Kotlin :
//!
//! ```kotlin
//! package fr.arsenal.fabbot
//! object Moteur {
//!     init { System.loadLibrary("fab_android") }
//!     @JvmStatic external fun demarrer(dossier: String): Int
//! }
//! ```
//!
//! Rend le port (ou -1 en cas d'erreur, détaillée dans le journal).

use jni::objects::{JClass, JString};
use jni::sys::jint;
use jni::JNIEnv;
use std::path::PathBuf;

#[no_mangle]
pub extern "system" fn Java_fr_arsenal_fabbot_Moteur_demarrer<'l>(
    mut env: JNIEnv<'l>,
    _classe: JClass<'l>,
    dossier: JString<'l>,
) -> jint {
    let dossier: String = match env.get_string(&dossier) {
        Ok(s) => s.into(),
        Err(e) => {
            crate::journal::ecrire(&format!("Dossier illisible : {e}"));
            return -1;
        }
    };
    let dossier = PathBuf::from(dossier);
    let textes = Some(crate::chemin_textes(&dossier));
    match std::panic::catch_unwind(|| crate::demarrer(&dossier, textes, crate::PORT)) {
        Ok(Ok(port)) => jint::from(port),
        Ok(Err(e)) => {
            crate::journal::ecrire(&format!("Démarrage impossible : {e}"));
            -1
        }
        Err(_) => -1,
    }
}
