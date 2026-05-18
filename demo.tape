Output demo.gif

Set Shell "bash"
Set FontSize 13
Set Width 900
Set Height 600
Set Theme "Catppuccin Mocha"
Set TypingSpeed 50ms
Set Padding 20

Hide
Type "export PATH=$HOME/.cargo/bin:/tmp/pawcrypt-target/release:$PATH"
Enter
Type "cd /tmp && rm -f secret.txt vault.paw decoy.txt real.txt"
Enter
Type `printf 'the real plans:\n  - steal all the yarn\n  - nap\n' > secret.txt`
Enter
Sleep 200ms
Show

Type "pawcrypt --help"
Enter
Sleep 2s

Type "pawcrypt seal secret.txt --out vault.paw --plausible"
Enter
Sleep 800ms
Type "groceries"
Enter
Sleep 200ms
Type "groceries"
Enter
Sleep 200ms
Type "hunter22"
Enter
Sleep 200ms
Type "hunter22"
Enter
Sleep 1500ms

Type "xxd vault.paw | head -4"
Enter
Sleep 1s

Type "pawcrypt open vault.paw --out decoy.txt"
Enter
Sleep 600ms
Type "groceries"
Enter
Sleep 800ms
Type "cat decoy.txt"
Enter
Sleep 2s

Type "pawcrypt open vault.paw --real --out real.txt"
Enter
Sleep 600ms
Type "hunter22"
Enter
Sleep 800ms
Type "cat real.txt"
Enter
Sleep 2s
