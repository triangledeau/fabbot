//! Messages et plantages : dans le journal d'Android (`adb logcat -s
//! fab-android`) sur le téléphone, dans la console sur PC.

use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Once;

/// Étiquette des messages dans le journal d'Android.
pub const ETIQUETTE: &str = "fab-android";

/// Écrit un message dans le journal.
pub fn ecrire(message: &str) {
    #[cfg(target_os = "android")]
    android::ecrire(message);
    #[cfg(not(target_os = "android"))]
    eprintln!("[{ETIQUETTE}] {message}");
}

/// Installe (une seule fois) le crochet qui note chaque plantage du Rust
/// (`panic`) dans le journal et à la fin de `fichier`
/// (`sauvegardes/plantages.txt`), avec la date.
pub fn noter_les_plantages(fichier: PathBuf) {
    static UNE_FOIS: Once = Once::new();
    UNE_FOIS.call_once(move || {
        let precedent = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let ou = info
                .location()
                .map(|l| format!(" ({}:{})", l.file(), l.line()))
                .unwrap_or_default();
            let quoi = info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "cause inconnue".into());
            let fil = std::thread::current()
                .name()
                .unwrap_or("sans nom")
                .to_string();
            let ligne = format!("Plantage dans le fil « {fil} » : {quoi}{ou}");
            ecrire(&ligne);
            let secondes = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            if let Some(d) = fichier.parent() {
                let _ = std::fs::create_dir_all(d);
            }
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&fichier)
            {
                let _ = writeln!(f, "[{secondes} s depuis 1970] {ligne}");
            }
            precedent(info);
        }));
    });
}

#[cfg(target_os = "android")]
mod android {
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int};

    #[link(name = "log")]
    extern "C" {
        fn __android_log_write(prio: c_int, tag: *const c_char, text: *const c_char) -> c_int;
    }

    /// Priorité « INFO » du journal d'Android.
    const INFO: c_int = 4;

    pub fn ecrire(message: &str) {
        let tag = CString::new(super::ETIQUETTE).unwrap_or_default();
        let texte = CString::new(message.replace('\0', " ")).unwrap_or_default();
        // SAFETY : deux chaînes C valides, terminées par un zéro, qui vivent
        // pendant tout l'appel.
        unsafe {
            __android_log_write(INFO, tag.as_ptr(), texte.as_ptr());
        }
    }
}
