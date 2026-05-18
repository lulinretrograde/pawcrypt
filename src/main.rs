mod crypto;
mod meow;
mod plausible;
mod stego;
mod ui;
mod volume;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pawcrypt", about = "deniable encryption owo~", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Encrypt a file into a dual-volume vault
    Seal {
        /// Input file to protect (goes in the real volume)
        input: PathBuf,
        /// Output vault file
        #[arg(short, long)]
        out: PathBuf,
        /// Use this file as decoy content instead of input
        #[arg(long, conflicts_with = "plausible")]
        decoy: Option<PathBuf>,
        /// Auto-generate convincing decoy content
        #[arg(long, conflicts_with = "decoy")]
        plausible: bool,
    },
    /// Decrypt a vault (decoy by default, --real for real volume)
    Open {
        /// Vault file to open
        vault: PathBuf,
        /// Open the real (hidden) volume
        #[arg(long)]
        real: bool,
        /// Output file for decrypted data
        #[arg(short, long)]
        out: PathBuf,
    },
    /// Hide a vault inside a PNG image (LSB steganography)
    Hide {
        /// Vault file to embed
        vault: PathBuf,
        /// Cover PNG image
        image: PathBuf,
        /// Output PNG with embedded vault
        #[arg(short, long)]
        out: PathBuf,
    },
    /// Extract a hidden vault from a PNG image
    Extract {
        /// PNG image containing a hidden vault
        image: PathBuf,
        /// Output vault file
        #[arg(short, long)]
        out: PathBuf,
    },
    /// Encode a vault as cat sounds (nya mew purr meow)
    Meow {
        /// Vault file to encode
        vault: PathBuf,
        /// Output text file of cat sounds
        #[arg(short, long)]
        out: PathBuf,
    },
    /// Decode a cat-sound file back into a vault
    Unmeow {
        /// Text file of cat sounds
        input: PathBuf,
        /// Output vault file
        #[arg(short, long)]
        out: PathBuf,
    },
}

fn main() -> Result<()> {
    ui::print_banner();

    let cli = Cli::parse();

    match cli.command {
        Command::Seal { input, out, decoy, plausible } => {
            cmd_seal(&input, &out, decoy.as_deref(), plausible)
        }
        Command::Open { vault, real, out } => cmd_open(&vault, real, &out),
        Command::Hide { vault, image, out } => cmd_hide(&vault, &image, &out),
        Command::Extract { image, out } => cmd_extract(&image, &out),
        Command::Meow { vault, out } => cmd_meow(&vault, &out),
        Command::Unmeow { input, out } => cmd_unmeow(&input, &out),
    }
}

fn cmd_seal(
    input: &std::path::Path,
    out: &std::path::Path,
    decoy_path: Option<&std::path::Path>,
    use_plausible: bool,
) -> Result<()> {
    let real_plaintext = std::fs::read(input)?;

    let decoy_plaintext = if use_plausible {
        ui::msg_info("generating plausible decoy content...");
        plausible::generate()
    } else if let Some(p) = decoy_path {
        std::fs::read(p)?
    } else {
        real_plaintext.clone()
    };

    ui::msg_sealing(out.to_string_lossy().as_ref());

    let decoy_pw = ui::prompt_password_confirm("decoy password")?;
    let real_pw = ui::prompt_password_confirm("real password")?;

    let ((decoy_salt, decoy_ct), (real_salt, real_ct)) =
        ui::with_spinner("encrypting (argon2 go brrr)...", || {
            crypto::encrypt_pair(&decoy_pw, &decoy_plaintext, &real_pw, &real_plaintext)
        })?;

    let layout = volume::VaultLayout::new(decoy_salt, real_salt, decoy_ct, real_ct);
    layout.write(out)?;

    ui::msg_sealed_ok(out.to_string_lossy().as_ref());
    Ok(())
}

fn cmd_open(vault: &std::path::Path, real: bool, out: &std::path::Path) -> Result<()> {
    let layout = volume::VaultLayout::read(vault)?;

    ui::msg_opening(real);

    let pw = ui::prompt_password(if real { "real password" } else { "decoy password" })?;

    let result = ui::with_spinner("decrypting...", || {
        if real {
            crypto::decrypt(&pw, &layout.real_salt, &layout.real_ct)
        } else {
            crypto::decrypt(&pw, &layout.decoy_salt, &layout.decoy_ct)
        }
    });

    match result {
        Ok(plaintext) => {
            std::fs::write(out, &plaintext)?;
            ui::msg_open_ok(out.to_string_lossy().as_ref(), real);
        }
        Err(e) => {
            ui::msg_wrong_password();
            return Err(e);
        }
    }

    Ok(())
}

fn cmd_hide(vault: &std::path::Path, image: &std::path::Path, out: &std::path::Path) -> Result<()> {
    ui::msg_info(&format!("hiding vault in {}...", image.display()));
    let vault_bytes = std::fs::read(vault)?;
    stego::hide(&vault_bytes, image, out)?;
    ui::msg_ok(&format!("vault hidden in {}! it's just a normal png nya~", out.display()));
    Ok(())
}

fn cmd_extract(image: &std::path::Path, out: &std::path::Path) -> Result<()> {
    ui::msg_info(&format!("extracting vault from {}...", image.display()));
    let vault_bytes = stego::extract(image)?;
    std::fs::write(out, &vault_bytes)?;
    ui::msg_ok(&format!("vault extracted to {} nya~", out.display()));
    Ok(())
}

fn cmd_meow(vault: &std::path::Path, out: &std::path::Path) -> Result<()> {
    ui::msg_info("encoding vault as cat sounds owo~");
    let vault_bytes = std::fs::read(vault)?;
    let encoded = meow::encode(&vault_bytes);
    std::fs::write(out, encoded.as_bytes())?;
    ui::msg_ok(&format!(
        "vault encoded to {} ({} sounds), totally innocent nya~",
        out.display(),
        vault_bytes.len() * 4
    ));
    Ok(())
}

fn cmd_unmeow(input: &std::path::Path, out: &std::path::Path) -> Result<()> {
    ui::msg_info("decoding cat sounds back to vault...");
    let text = std::fs::read_to_string(input)?;
    let vault_bytes = meow::decode(&text)?;
    std::fs::write(out, &vault_bytes)?;
    ui::msg_ok(&format!("vault decoded to {} nya~", out.display()));
    Ok(())
}
