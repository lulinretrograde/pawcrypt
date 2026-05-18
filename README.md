# 🐱 pawcrypt ✨

## Cryptographically Deniable, Objectively Adorable

The *purr*-fect way to hide secrets that can't even be *paw*-ven to exist, because what vault? There is no vault. It's just random bytes, nya~

![version](https://img.shields.io/badge/version-0.2.0-ff69b4) ![Argon2id](https://img.shields.io/badge/KDF-Argon2id-b482ff) ![AES-256-GCM](https://img.shields.io/badge/cipher-AES--256--GCM-ff82c8) ![Meow Rating](https://img.shields.io/badge/meow%20rating-nya%2F10-ff69b4) ![uwu factor](https://img.shields.io/badge/uwu%20factor-extremely%20high-b482ff)

> [!IMPORTANT]
> This is *real cryptography* with a *real security guarantee*, not just aesthetic. The deniability is **cryptographic**, not theatrical. Two independent AES-256-GCM volumes, zero common headers, zero magic bytes. An adversary with your vault file and no password cannot prove a second volume exists. pinky promise 🩷

---

## What's This Adorable Little Thing? 🐾

It's what happens when someone reads the VeraCrypt hidden volume spec, falls down a rabbit hole and decides these two things belong together.

One file. Two passwords. Two completely independent encrypted volumes baked inside, a decoy up front and your real secret in the back. To anyone without a password, the whole file looks like random noise. Under the uwu exterior: Argon2id key derivation (64 MiB, 3 iterations, the one that wins password-hashing competitions), AES-256-GCM authenticated encryption, and zlib compression before every encrypt pass. Both volumes get independent salts and nonces with no shared state between them.

> "It looks like random bytes to me, officer." - *You, truthfully*

> [!NOTE]
> There is no bit in the file that says "a second volume exists." Even the salt positions are deterministic but reveal nothing without the password. It's the same idea as VeraCrypt's hidden volumes, but make it pink.

---

## How To Use This Fluffy Tool 🔐

### Sealing Your Secrets (`seal`)

Basic seal, both volumes contain the same file:

```bash
pawcrypt seal secret.txt --out vault.paw
```

With auto-generated innocent decoy content (recommended for actual deniability):

```bash
pawcrypt seal secret.txt --out vault.paw --plausible
```

*A random shopping list, diary entry, or recipe gets picked as decoy content. If anyone opens the decoy volume they see "need to get: bananas, yogurt, orange juice..." Boring, convincing, nothing to see here uwu*

With your own decoy file:

```bash
pawcrypt seal real_secret.txt --out vault.paw --decoy innocent_notes.txt
```

> [!WARNING]
> Use different passwords for decoy and real volumes. Same password twice is just a very elaborate way to encrypt something twice. Pick a decoy password you can actually remember, because under duress is not the time to go blank.

### Opening Your Vault (`open`)

Open the decoy volume (what you show people):

```bash
pawcrypt open vault.paw --out decoy_content.txt
```

Open the real volume (what only you know about):

```bash
pawcrypt open vault.paw --real --out real_content.txt
```

*Both commands look identical from the outside. There is no `--real` flag stored anywhere in the file. Nobody can tell which one you ran.*

### Hiding In Plain Sight: PNG Steganography (`hide` / `extract`)

Your vault file is already indistinguishable from random bytes, but why not also hide it inside a cat picture?

```bash
# tuck the vault inside a PNG
pawcrypt hide vault.paw cat_photo.png --out totally_normal_cat.png

# get it back out
pawcrypt extract totally_normal_cat.png --out recovered.paw
```

*Visual difference per pixel: ±1 in one colour channel. Imperceptible to the human eye. Detectable only if someone is already looking for it.*

> [!CAUTION]
> Don't re-compress or re-encode the PNG after hiding. JPEG conversion or PNG re-encoding will destroy the LSB data. Keep it as a lossless PNG, treat it like a normal photo, don't do anything weird to it. Just a cat photo. Normal. Nothing going on.

A 500x500 PNG holds roughly 125 KB of vault. Scale up for larger secrets.

### The Meow Layer: Cat Sound Encoding (`meow` / `unmeow`)

Your vault bytes can be encoded as cat vocalizations:

```bash
pawcrypt meow vault.paw --out meows.txt
pawcrypt unmeow meows.txt --out vault.paw
```

Output looks like this:

```
meow nya purr mew nya nya meow purr mew mew nya purr meow nya purr mew
purr meow nya nya mew mew meow meow nya purr mew nya meow purr mew nya
nya meow purr mew purr meow nya nya mew mew meow meow nya purr mew mew
```

*Just looks like you let your cat write a message. Which is a valid explanation.*

Each byte becomes 4 cat sounds (base-4, 2 bits per sound, MSB first):

| bits | sound |
|------|-------|
| `00` | `nya` |
| `01` | `mew` |
| `10` | `purr`|
| `11` | `meow`|

> [!NOTE]
> The meow layer is transport encoding, not crypto. It hides the *appearance* of ciphertext but provides no additional cryptographic security. The security lives in the vault. The meows are just aesthetic. Beautiful, beautiful aesthetic.

---

## The Complete Command Reference 🐾

```
deniable encryption owo~

Usage: pawcrypt <COMMAND>

Commands:
  seal     Encrypt a file into a dual-volume vault
  open     Decrypt a vault (decoy by default, --real for real volume)
  hide     Hide a vault inside a PNG image (LSB steganography)
  extract  Extract a hidden vault from a PNG image
  meow     Encode a vault as cat sounds (nya mew purr meow)
  unmeow   Decode a cat-sound file back into a vault
  help     Print help

seal options:
  <INPUT>          File to protect (goes in the real volume)
  --out <OUT>      Output vault file
  --plausible      Auto-generate convincing decoy content
  --decoy <FILE>   Use this file as decoy content (exclusive with --plausible)

open options:
  <VAULT>          Vault file to open
  --real           Open the real (hidden) volume
  --out <OUT>      Output file for decrypted data
```

---

## How The Vault Actually Works 🔮

### What Random Bytes?

```
bytes  0-31    decoy salt  (32 bytes, random)
bytes 32-63    real  salt  (32 bytes, random)
bytes 64..N    decoy ciphertext (nonce || ct || tag)
bytes N..end   real  ciphertext (nonce || ct || tag)
```

No magic bytes. No version header. No length fields. Both ciphertexts are always exactly the same byte count, with the shorter plaintext padded to match the longer one inside the AES-GCM envelope (so the padding is authenticated and the real length is invisible without the password). Salt positions are deterministic so you can re-derive the key, but the 32 random salt bytes reveal nothing on their own. The AES-GCM authentication tag covers every byte, so a wrong password gives a clean error with no partial decryption, no oracle.

### Key Derivation

Each volume gets its own independent 32-byte random salt fed into Argon2id at 64 MiB / 3 iterations / parallelism 1. Keys are zeroed from memory immediately after use. The spinner you see during `seal` and `open` is Argon2 actually working. That time cost is intentional and is exactly what makes brute force expensive.

> [!IMPORTANT]
> Argon2id at 64 MiB means any attacker trying passwords must allocate 64 MiB of RAM *per guess*. Against a 12-character random passphrase this is thousands of years on modern hardware. Pick a good passphrase. Don't use "password123" for your real volume. We will not be held responsible for the consequences.

---

## Actual Testimonials From Satisfied Users 📝

> "nya nya purr meow nya mew purr nya meow meow" - **Mittens**, Chief Privacy Officer at MeowSoft Inc.

> "the decoy volume had a grocery list. they found nothing." - **Anonymous**, definitely not a spy

> "i hid the vault in a PNG of my cat and sent it to myself as a 'backup'. my cat is now a government secret." - **Cat Lady #7**

> "I used --plausible and the decoy had a lemon drizzle cake recipe. It was actually a good recipe. I made it. 10/10." - **Verified User**

> [!CAUTION]
> May cause: saying "nya~" out loud while encrypting, trusting your cat more than your operating system, an uncontrollable urge to put everything in a `.paw` file, and strongly-held opinions about Argon2id parameter tuning. Side effects are considered features.

---

## Security Notes (Read These, Seriously) 🔒

> [!WARNING]
> **Password strength is everything.** No key files. No recovery mechanism. No "forgot password." Lose both passwords and the data is gone. The Argon2id parameters make brute force expensive but not impossible against weak passwords. Use a long, random passphrase.

> [!CAUTION]
> **The decoy password must be one you can plausibly claim is your only password.** Under pressure from a technically sophisticated party, they may count the bytes in the file and infer a second volume could exist. The layout is designed to make this unprovable, but social engineering is not a crypto problem.

> [!NOTE]
> **Meow encoding and PNG steganography are not cryptographic security.** They hide the existence of a vault from casual inspection. A forensic examiner looking specifically for LSB-modified PNGs or cat-word patterns will find them. The crypto layer protects the content regardless of whether the container is found.

---

## License

MIT, because secrets should be free (the software, not necessarily the secrets themselves).

---

*"In a world of boring encryption tools, be an argon2id-hardened cat with forged plausible deniability."* 🐾
