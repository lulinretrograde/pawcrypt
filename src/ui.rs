use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::time::Duration;

const CAT_ART: &str = r#"
  /\_____/\
 /  o   o  \
( ==  ^  == )
 )         (
(           )
 \  ||  ||  /
  \_||__||_/
"#;

pub fn print_banner() {
    println!("{}", CAT_ART.truecolor(255, 182, 255));
    println!(
        "{}  {}",
        "pawcrypt".truecolor(255, 105, 180).bold(),
        "v0.2.0 — deniable encryption owo~".truecolor(180, 130, 255)
    );
    println!();
}

/// Run `f` while showing an animated spinner. Spinner ticks in a background
/// thread via indicatif, so f() can be blocking (e.g. Argon2 derivation).
pub fn with_spinner<F, T>(msg: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.magenta} {msg}")
            .expect("spinner template valid"),
    );
    pb.set_message(format!("{}", msg.truecolor(200, 160, 255)));
    pb.enable_steady_tick(Duration::from_millis(80));
    let result = f();
    pb.finish_and_clear();
    result
}

pub fn msg_sealing(path: &str) {
    println!(
        "{} {}",
        ">>".truecolor(255, 105, 180).bold(),
        format!("sealing ur secrets into {path} owo~").truecolor(255, 182, 255)
    );
}

pub fn msg_sealed_ok(path: &str) {
    println!(
        "{} {}",
        "✓".truecolor(105, 255, 180).bold(),
        format!("vault written to {path}! ur secrets r safe nya~").truecolor(255, 182, 255)
    );
}

pub fn msg_opening(real: bool) {
    let volume = if real { "real volume" } else { "decoy volume" };
    println!(
        "{} {}",
        ">>".truecolor(255, 105, 180).bold(),
        format!("unlocking {volume}...").truecolor(255, 182, 255)
    );
}

pub fn msg_open_ok(path: &str, real: bool) {
    let flavor = if real {
        "real volume unlocked! nya~".to_string()
    } else {
        "decoy volume unlocked! nothing to see here uwu".to_string()
    };
    println!(
        "{} {} → {}",
        "✓".truecolor(105, 255, 180).bold(),
        flavor.truecolor(255, 182, 255),
        path.truecolor(180, 130, 255)
    );
}

pub fn msg_wrong_password() {
    eprintln!(
        "{} {}",
        "✗".truecolor(255, 80, 80).bold(),
        "wrong password detected!! >:3".truecolor(255, 120, 120)
    );
}

pub fn msg_info(text: &str) {
    println!(
        "  {} {}",
        "~".truecolor(180, 130, 255),
        text.truecolor(200, 160, 255)
    );
}

pub fn msg_ok(text: &str) {
    println!(
        "{} {}",
        "✓".truecolor(105, 255, 180).bold(),
        text.truecolor(255, 182, 255)
    );
}

pub fn prompt_password(label: &str) -> anyhow::Result<Vec<u8>> {
    let prompt = format!(
        "{} {}: ",
        ">>".truecolor(255, 105, 180).bold(),
        label.truecolor(255, 182, 255)
    );
    let pw = rpassword::prompt_password(prompt)?;
    Ok(pw.into_bytes())
}

pub fn prompt_password_confirm(label: &str) -> anyhow::Result<Vec<u8>> {
    let pw1 = prompt_password(label)?;
    let pw2 = prompt_password(&format!("{label} (confirm)"))?;
    if pw1 != pw2 {
        anyhow::bail!("passwords didn't match >:3");
    }
    Ok(pw1)
}
