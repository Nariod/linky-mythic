# Changelog – linky-mythic

> **Format**: [Semantic Versioning 2.0.0](https://semver.org/)
> **Convention**: [Conventional Commits](https://www.conventionalcommits.org/)

---

## [Unreleased]

### ✅ Ajouté
- **ROADMAP.md**: Feuille de route complète avec priorités, timeline et métriques de succès
- **Lien croisé**: Référence à ROADMAP.md dans TODO.md pour une vision stratégique

### 📝 Documentation
- Ajout de la section "Feuille de route globale" dans TODO.md pointant vers ROADMAP.md
- Création d'une structure de suivi des progrès par phase et par priorité

---

## [0.2.0] – 2026-04-15 (Beta - Live Tested)

### ✅ Fonctionnalités
- **21/21 commandes validées** en live contre Mythic v3.4.32
  - Commandes cross-platform: `whoami`, `pwd`, `pid`, `info`, `ls`, `cd`, `shell`, `ps`, `netstat`, `sleep`, `killdate`, `download`, `upload`, `cp`, `mv`, `rm`, `mkdir`, `execute`, `exit`
  - Commandes Windows: `inject`, `integrity`, `cmd`, `powershell`
- **Indirect Syscalls** (Windows) via [syscalls-rs](https://github.com/Nariod/syscalls-rs)
- **Protocole Mythic complet**: AES-256-CBC + HMAC-SHA256, chunked file transfers

### 🔧 Améliorations
- **Réduction taille binaire**: 4.5 MB → **1.9 MB** (Linux) grâce à `ureq` + `ring`
- **OPSEC**: 
  - String obfuscation avec `obfstr` pour les actions Mythic sensibles
  - `zeroize` pour effacer les clés en mémoire
  - `RUSTFLAGS="-C debuginfo=0 --remap-path-prefix=..."` pour éviter les leaks de chemins
- **Builder Go**: Support complet des paramètres Mythic (AESPSK, callback_host, etc.)

### 🐛 Corrections
- Fix `upload.go`: `TaskFunctionParseArgString` pour gérer le JSON input
- Fix `inject.go`: Ajout de `TaskFunctionParseArgString` (manquant)
- Fix `sleep.go`: Support du format `seconds jitter%`
- Fix `cd`: Retourne maintenant `[+] /nouveau/chemin`
- Fix `whoami`: Fallback sur `/proc/sys/kernel/hostname` pour Fedora/systemd

### 📦 Build
- Migration de `reqwest` → `ureq 3` pour réduire la taille
- Pinned Rust version: `rust:1.86` dans Dockerfile
- Support cross-compilation: Linux (musl), Windows (mingw-w64)

### 🧪 Tests
- **9/9 tests unitaires** passent (Go + Rust)
- Validation live: Callback Linux et Windows vérifiés
- Build payload: Linux (~1.9 MB), Windows (~1.6 MB)

---

## [0.1.0] – 2026-03-01 (MVP)

### ✅ Fonctionnalités
- **16 commandes de base** implémentées
- **Crypto Mythic**: AES-256-CBC + HMAC-SHA256 compatible
- **Upload/Download**: Protocole chunked via Mythic file store
- **Process/File Browser**: Intégration JSON pour l'UI Mythic

### 📦 Infrastructure
- Structure standard Mythic: `Payload_Type/linky/`
- Dockerfile multi-stage: Go builder + Rust toolchain
- Workspace Cargo: `common/`, `linux/`, `windows/`, `osx/`

---

## [0.0.1] – 2026-01-15 (Pre-Alpha)

### 🚀 Initialisation
- Migration du code depuis `linky/` (Rust C2 framework)
- Adaptation au format Mythic (AES256_HMAC)
- Premier callback réussi en lab

---

## 📊 Statistiques par Version

| Version | Date | Commandes | Plateformes | Taille Linux | Statut |
|---------|------|-----------|-------------|--------------|--------|
| 0.0.1 | 2026-01-15 | 5 | Linux | ~5 MB | Pre-Alpha |
| 0.1.0 | 2026-03-01 | 16 | Linux/Windows | ~4.5 MB | MVP |
| 0.2.0 | 2026-04-15 | 21 | Linux/Windows | **1.9 MB** | Beta |
| Unreleased | - | 21+ | Linux/Windows/macOS* | < 2 MB | **En développement** |

*macOS: Support en cours (Phase P1)

---

## 🔮 Roadmap (Voir [ROADMAP.md](ROADMAP.md))

| Version | Cible | Priorités |
|---------|-------|-----------|
| **0.3.0** | Juin 2026 | macOS, ARM64, AMSI Bypass |
| **0.4.0** | Sept 2026 | Sleep Obfuscation, Modules Dynamiques |
| **1.0.0** | 2027 | Toutes fonctionnalités + documentation |

---

## 📝 Conventions

### Types de changements
- **✅ Ajouté**: Nouvelles fonctionnalités
- **🔧 Améliorations**: Optimisations, refactoring
- **🐛 Corrections**: Bug fixes
- **📦 Build**: Changements d'infrastructure de build
- **🧪 Tests**: Ajout/modification de tests
- **📝 Documentation**: Mises à jour de documentation

### Format des commits
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Exemples:
- `feat(commands): add cp/mv/rm/mkdir commands`
- `fix(builder): handle JSON input in upload.go`
- `perf(binary): replace reqwest with ureq (-58% size)`
- `docs: update ROADMAP.md with Q3 priorities`

---

## 🔗 Liens
- [ROADMAP.md](ROADMAP.md) – Feuille de route détaillée
- [TODO.md](TODO.md) – Plan de migration et audit technique
- [README.md](README.md) – Documentation principale
- [TEST_PROCEDURE.md](TEST_PROCEDURE.md) – Procédures de test
