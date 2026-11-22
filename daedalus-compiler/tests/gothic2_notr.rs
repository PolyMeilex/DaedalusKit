use std::{io::Cursor, path::PathBuf};

use daedalus_compiler::{CompilationUnit, SourceFile};
use dat_file::DatFile;

fn guess_game_dir() -> Option<PathBuf> {
    let mut home = PathBuf::from(std::env::var("HOME").ok()?);
    home.push(".local/share/Steam/steamapps/common/Gothic II");
    if home.exists() { Some(home) } else { None }
}

fn check_unit(unit: CompilationUnit, src: &str, dat: &str) {
    let Some(game) = std::env::var("GOTHIC2_DIR")
        .map(PathBuf::from)
        .ok()
        .or(guess_game_dir())
    else {
        eprintln!("GOTHIC2_DIR not fond");
        return;
    };

    let scripts = game.join("_work/Data/Scripts");
    let original_dat = scripts.join("_compiled").join(dat);

    let sources = SourceFile::read_src(scripts.join(src)).unwrap();
    let compiled = match daedalus_compiler::compile(unit, &sources) {
        Ok(compiled) => compiled,
        Err(errors) => panic!("failed to compile, with {} errors", errors.len()),
    };

    let expected = std::fs::read(original_dat).unwrap();

    let decode = |bytes: &[u8]| DatFile::decode(&mut Cursor::new(bytes)).unwrap();

    let expected = decode(&expected);
    let compiled = decode(&compiled);

    dat_file::diff(compiled, expected);
}

#[test]
fn gothic() {
    check_unit(CompilationUnit::Gothic, "Content/Gothic.src", "GOTHIC.DAT");
}

#[test]
fn fight() {
    check_unit(CompilationUnit::Fight, "Content/Fight.src", "FIGHT.DAT");
}

#[test]
fn camera() {
    check_unit(CompilationUnit::Camera, "System/Camera.src", "CAMERA.DAT");
}

#[test]
fn menu() {
    check_unit(CompilationUnit::Menu, "System/Menu.src", "MENU.DAT");
}

#[test]
fn music() {
    check_unit(CompilationUnit::Music, "System/Music.src", "MUSIC.DAT");
}

#[test]
fn sfx() {
    check_unit(CompilationUnit::Sfx, "System/SFX.src", "SFX.DAT");
}

#[test]
fn particle_fx() {
    check_unit(
        CompilationUnit::ParticleFx,
        "System/ParticleFX.src",
        "PARTICLEFX.DAT",
    );
}

#[test]
fn visual_fx() {
    check_unit(
        CompilationUnit::VisualFx,
        "System/VisualFX.src",
        "VISUALFX.DAT",
    );
}
