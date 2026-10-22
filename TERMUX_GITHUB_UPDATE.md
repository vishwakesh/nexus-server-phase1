# Termux → GitHub: publish Phase 2 without replacing Phase 1 main

This ZIP contains source files only, not `.git` metadata. Keep Phase 1 `main` unchanged and push the new version to a separate branch.

## 1. Extract on the phone

```bash
termux-setup-storage
pkg install git unzip -y
mkdir -p ~/projects/nexus-server-phase2-source
unzip ~/storage/downloads/nexus-server-phase2.zip -d ~/projects/nexus-server-phase2-source
cd ~/projects/nexus-server-phase2-source/nexus-server-phase2
```

If your downloaded file has another name, use that exact name in the `unzip` command.

## 2. Copy into your existing Git checkout on a new branch

The existing repository checkout is expected at `~/projects/nexus-server-phase1`. If it differs, adjust that path.

```bash
cd ~/projects/nexus-server-phase1
git switch -c phase2-auth
```

Copy the new files over the checkout **without copying the ZIP's `.git` data**:

```bash
cp -a ~/projects/nexus-server-phase2-source/nexus-server-phase2/. .
git status --short
```

Inspect the changed files. Do not add `.env`, private keys, access tokens, or database files. This source package includes `.env.example` only.

## 3. Commit and push

```bash
git add Cargo.toml .gitignore BUILD_STATUS.md README.md .env.example src frontend migrations docs scripts deploy TERMUX_GITHUB_UPDATE.md
git status --short
git commit -m "Add NEXUS Phase 2 authentication foundation"
git push -u origin phase2-auth
```

Use browser-based GitHub CLI login (`gh auth login`) if prompted. Never paste access tokens into a chat.

## 4. VPS deployment comes after the branch is pushed

On the VPS, fetch and inspect the new branch before deploying it. Do not pull this branch into `/opt/nexus-server-phase1`, and do not use the Phase 1 database. Build Phase 2 in `/opt/nexus-server-phase2`, use the dedicated data directory `/var/lib/nexus-server-phase2/data`, and port `3001`.
