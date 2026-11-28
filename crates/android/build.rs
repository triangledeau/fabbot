//! Prépare les fichiers embarqués dans la bibliothèque Android :
//!
//! - les decks du projet (`data/decks/*.txt`, sans les sous-dossiers) : un
//!   deck ajouté par un autre agent arrive ici tout seul ;
//! - le vrai dos de carte (`data/outils/dos-de-carte.webp`, `.png` ou
//!   `.jpg`), **s'il est présent** sur la machine qui compile. Il n'est pas
//!   dans le dépôt (fichier privé) : sur GitHub, il viendra d'un secret.
//!
//! Le résultat est écrit dans `$OUT_DIR/embarques.rs`, lu par `donnees.rs`.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let racine = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let decks = racine.join("data/decks");
    println!("cargo:rerun-if-changed={}", decks.display());

    let mut fichiers: Vec<PathBuf> = std::fs::read_dir(&decks)
        .expect("dossier data/decks introuvable")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "txt"))
        .collect();
    fichiers.sort();

    let mut code = String::from("/// Les decks du projet : (nom du fichier, contenu).\n");
    code.push_str("pub static DECKS: &[(&str, &[u8])] = &[\n");
    for f in &fichiers {
        println!("cargo:rerun-if-changed={}", f.display());
        let nom = f.file_name().unwrap().to_string_lossy();
        writeln!(code, "    ({nom:?}, include_bytes!({:?})),", chemin(f)).unwrap();
    }
    code.push_str("];\n\n");

    let outils = racine.join("data/outils");
    let mut dos = None;
    for (ext, type_contenu) in [
        ("webp", "image/webp"),
        ("png", "image/png"),
        ("jpg", "image/jpeg"),
    ] {
        let f = outils.join(format!("dos-de-carte.{ext}"));
        println!("cargo:rerun-if-changed={}", f.display());
        if dos.is_none() && f.is_file() {
            dos = Some((f, type_contenu));
        }
    }
    code.push_str("/// Le vrai dos de carte (contenu, type), s'il était là à la compilation.\n");
    match dos {
        Some((f, t)) => writeln!(
            code,
            "pub static DOS_DE_CARTE: Option<(&[u8], &str)> = Some((include_bytes!({:?}), {t:?}));",
            chemin(&f)
        )
        .unwrap(),
        None => code.push_str("pub static DOS_DE_CARTE: Option<(&[u8], &str)> = None;\n"),
    }

    let sortie = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("embarques.rs");
    std::fs::write(sortie, code).unwrap();
}

/// Chemin absolu, pour `include_bytes!`.
fn chemin(f: &Path) -> String {
    f.canonicalize()
        .unwrap_or_else(|_| f.to_path_buf())
        .to_string_lossy()
        .into_owned()
}
