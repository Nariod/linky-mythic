# linky-mythic - Feuille de Route

> **Mythic Payload Type en Rust** - Implants natifs pour Linux, Windows et macOS
> **Statut actuel**: Beta (Live-tested contre Mythic v3.4.32 - Avril 2026)
> **Dernière mise à jour**: Juillet 2026

---

## 📊 Statut Global

| Métrique | Valeur | Statut |
|----------|--------|--------|
| Version Mythic compatible | v3.4.32 | ✅ |
| Taille binaire Linux | ~1.9 MB | ✅ |
| Taille binaire Windows | ~1.6-2 MB | ✅ |
| Commandes implémentées | 21/21 | ✅ |
| Tests unitaires | 9/9 | ✅ |
| Callback live vérifié | Linux + Windows | ✅ |
| Crypto | AES-256-CBC + HMAC-SHA256 | ✅ |
| OPSEC | obfstr, zeroize, strip, LTO | ✅ Partiel |

---

## ✅ Phases Terminees

### 🎯 Core (Phases 0-6)

| Phase | Description | Statut | Validation |
|-------|-------------|--------|------------|
| **0** | Structure du dépôt (`Payload_Type/linky/`) | ✅ | `mythic-cli install` fonctionnel |
| **1** | Migration des crates Rust depuis Linky | ✅ | Workspace Cargo opérationnel |
| **2** | Nettoyage du format Mythic (`lib.rs`) | ✅ | Crypto AES256_HMAC compatible |
| **3** | Builder Go complet | ✅ | `cargo build` + encryption callback |
| **4** | Définitions des 21 commandes en Go | ✅ | shell, inject, download, etc. |
| **5** | Configuration HTTPS + profil HTTP C2 | ✅ | TLS via `ureq` + `rustls` |
| **6** | Tests end-to-end contre Mythic | ✅ | Callback Linux/Windows vérifié |

### 🔧 Améliorations (Phases 7-12)

| Phase | Description | Statut |
|-------|-------------|--------|
| **7** | Upload/Download via Mythic file store | ✅ | Protocole chunked implémenté |
| **8** | Hardening OPSEC (zeroize, jitter, retry) | ✅ | Clés effacées en mémoire |
| **9** | Process/File Browser (JSON structuré) | ✅ | Intégration Mythic UI |
| **10** | Commandes manquantes (cp, mv, rm, mkdir, execute) | ✅ | Parité avec Hannibal |
| **11** | Réduction taille binaire (reqwest → ureq) | ✅ | **1.9 MB** (vs 4.5 MB) |
| **12** | OPSEC hardening (obfstr, debuginfo=0) | ✅ | Strings sensibles obfusquées |

### 🧪 Validation (Phases 13-17)

| Phase | Description | Statut |
|-------|-------------|--------|
| **13.5** | Indirect Syscalls (Windows) | ✅ | Intégration `syscalls-rs` |
| **16** | Restructuration dépôt + fixes Dockerfile | ✅ | Compatible `mythic-cli` |
| **17** | Audit complet + tests live | ✅ | 21/21 commandes validées |

---

## 🚀 Feuille de Route Future

### 🔥 Priorité Haute (0-3 mois)

| ID | Tâche | Description | Impact | Dépendances | Statut |
|----|-------|-------------|--------|-------------|--------|
| **P1** | macOS Support Complet | Intégrer `osxcross` dans le Dockerfile + cibles `x86_64-apple-darwin`/`aarch64-apple-darwin` | 🎯 Cross-platform | Phase 13 | ⏳ À faire |
| **P2** | ARM64 Support | Ajouter `aarch64-unknown-linux-musl` et `aarch64-apple-darwin` | 🌍 IoT/Cloud | P1 | ⏳ À faire |
| **P3** | AMSI/ETW Bypass (Windows) | Implémenter en Rust via `syscalls-rs` ou `ntapi` | 🛡️ Évasion EDR | Indirect Syscalls | ⏳ À faire |
| **P4** | User-Agent Configurable | Paramètre de build pour éviter la détection | 🕵️ OPSEC | Builder Go | ⏳ À faire |
| **P5** | Conditional Command Compilation | Features Cargo pour désactiver des commandes (ex: `--no-inject`) | ⚙️ Flexibilité | Refactor Cargo | ⏳ À faire |

### 📈 Priorité Moyenne (3-6 mois)

| ID | Tâche | Description | Impact | Statut |
|----|-------|-------------|--------|--------|
| **M1** | Sleep Obfuscation (Windows) | Implémenter Ekko-style (RC4 `.text` encryption) ou Foliage (APC-based) | 🛡️ OPSEC | ⏳ À faire |
| **M2** | Dynamic Module Loading | Équivalent HBIN (chargement de modules Rust `.so`/`.dll`) | 🔌 Extensibilité | ⏳ À faire |
| **M3** | SOCKS Proxy | Pivoting réseau via Mythic | 🌐 Post-exploitation | ⏳ À faire |
| **M4** | Process Browser Structuré | Sortie JSON pour `ps` (intégration Mythic UI) | 📊 UX | ⏳ À faire |
| **M5** | File Browser Structuré | Sortie JSON pour `ls` (métadonnées fichiers) | 📁 UX | ⏳ À faire |
| **M6** | Tests d'Intégration CI | Docker-in-Docker avec Mythic + HTTP C2 profile | ✅ Qualité | ⏳ À faire |

### 🌌 Priorité Basse (6-12 mois)

| ID | Tâche | Description | Impact | Statut |
|----|-------|-------------|--------|--------|
| **L1** | Hugo Documentation | Site de doc Mythic-compatible | 📖 Maintenance | ⏳ À faire |
| **L2** | Keylogging (Linux/Windows) | Capture des entrées clavier | 🎹 Post-exploitation | ⏳ À faire |
| **L3** | Credential Access | Dump de mots de passe (Mimikatz-like) | 🔑 Post-exploitation | ⏳ À faire |
| **L4** | UPX Compression | Réduction supplémentaire de la taille (optionnel) | ⚖️ Trade-off AV | ⏳ À faire |
| **L5** | Custom Crypto Primitives | Remplacer `aes`/`cbc`/`hmac` par des implémentations inline | 📉 Taille | ⏳ À faire |

---

## 📅 Timeline Estimée

| Période | Objectifs |
|---------|-----------|
| **Avril-Juin 2026** | ✅ Beta Stable (21 commandes, tests live) |
| **Juin-Sept 2026** | **P1-P5** (macOS, ARM64, AMSI Bypass, OPSEC) |
| **Sept-Déc 2026** | **M1-M6** (Sleep Obfuscation, Modules, Proxy) |
| **2027** | **L1-L5** (Documentation, Keylogging, etc.) |

---

## 🎯 Objectifs par Release

| Version | Cible | Fonctionnalités | Statut |
|---------|-------|------------------|--------|
| **v0.1.0** | MVP | 16 commandes de base + crypto Mythic | ✅ Sorti |
| **v0.2.0** | Beta | 21 commandes + indirect syscalls + tests live | ✅ Sorti |
| **v0.3.0** | **Prochaine** | macOS + ARM64 + AMSI Bypass | 🚧 En cours |
| **v0.4.0** | Future | Sleep Obfuscation + Modules Dynamiques | 📅 Planifié |
| **v1.0.0** | Stable | Toutes fonctionnalités + documentation complète | 🎯 Objectif |

---

## 📈 Métriques de Succès

| Métrique | Cible | Actuel |
|----------|-------|--------|
| Taille binaire Linux | < 2 MB | ✅ 1.9 MB |
| Taille binaire Windows | < 2.5 MB | ✅ 1.6-2 MB |
| Commandes supportées | 25+ | ✅ 21 |
| Plateformes | 3/3 (Linux/Win/macOS) | ⚠️ 2/3 (macOS en cours) |
| Tests unitaires | 100% couverture | ✅ 9/9 |
| Tests d'intégration | CI automatisée | ⚠️ Partiel (Phase 15) |
| OPSEC Score | 10/10 | ⚠️ 7/10 (Sleep Obfuscation manquant) |

---

## 🛠️ Prochaines Étapes Immédiates

### 1. 🔥 Terminer macOS Support (Phase 13)
- [ ] Ajouter `osxcross` au Dockerfile
- [ ] Tester la cross-compilation `x86_64-apple-darwin`
- [ ] Valider le callback macOS contre Mythic

### 2. 🔥 Implémenter AMSI/ETW Bypass (Priorité P3)
- [ ] Rechercher des crates Rust existantes (`amsi-bypass-rs`?)
- [ ] Intégrer dans `nt_inject.rs` ou nouveau module

### 3. 🔥 Conditional Compilation (Priorité P5)
- [ ] Définir les features Cargo (`cmd-inject`, `cmd-shellcode`, etc.)
- [ ] Mettre à jour `builder.go` pour passer les features sélectionnées

### 4. 📊 Améliorer la CI (Priorité M6)
- [ ] Finaliser `.github/workflows/test.yml` (Docker-in-Docker)
- [ ] Ajouter des tests de build pour toutes les cibles

---

## 📌 Ressources Clés

| Ressource | Lien | Usage |
|-----------|------|-------|
| Dépôt | [Nariod/linky-mythic](https://github.com/Nariod/linky-mythic) | Code source |
| Mythic | [its-a-feature/Mythic](https://github.com/its-a-feature/Mythic) | Framework C2 |
| HTTP C2 Profile | [MythicC2Profiles/http](https://github.com/MythicC2Profiles/http) | Transport HTTPS |
| syscalls-rs | [Nariod/syscalls-rs](https://github.com/Nariod/syscalls-rs) | Indirect Syscalls |
| Documentation | `TODO.md` | Détail technique |
| Tests | `run_tests.sh` | Validation locale |

---

## 🚨 Risques et Blocages

| Risque | Impact | Mitigation | Statut |
|--------|--------|------------|--------|
| macOS Cross-Compilation | Bloque la release v0.3.0 | Utiliser `osxcross` + SDK Apple | ⚠️ En cours |
| AMSI Bypass | Détection par EDR | Tests en environnement isolé | ⚠️ À démarrer |
| Taille Binaire | > 2 MB = suspect | Prioriser `ureq` + `ring` | ✅ Résolu |
| Compatibilité Mythic | Breaking changes | Suivre les releases Mythic | ✅ Suivi |
| SELinux | Bloque les builds Docker | `chcon -Rt svirt_sandbox_file_t` | ✅ Documenté |

---

## 📝 Checklist pour Contributeurs

- [ ] Lire **`TODO.md`** et **`ROADMAP.md`** en entier
- [ ] Respecter les conventions **`SKILLS.md`** (Clean Code)
- [ ] Valider avec `cargo test --workspace` + `go build ./...`
- [ ] Tester contre une instance Mythic locale
- [ ] Mettre à jour **`TODO.md`** et **`ROADMAP.md`** après chaque phase

---

## 🔄 Processus de Mise à Jour

1. **Avant de commencer une tâche**:
   - Vérifier la section correspondante dans `ROADMAP.md`
   - Marquer la tâche comme "En cours" (⏳)
   - Créer une branche `vibe/<nom-tache>-<date>`

2. **Pendant le développement**:
   - Suivre les conventions `SKILLS.md`
   - Documenter les changements dans `CHANGELOG.md` (si applicable)
   - Valider avec les tests existants

3. **Après completion**:
   - Marquer la tâche comme ✅ dans `ROADMAP.md`
   - Mettre à jour le statut dans `TODO.md` si nécessaire
   - Ouvrir une PR avec référence à la tâche dans la roadmap

---

## 📞 Contact

Pour toute question ou contribution, ouvrir une issue sur [GitHub](https://github.com/Nariod/linky-mythic).

---

> **💡 Note**: Cette feuille de route est **dynamique** et sera mise à jour régulièrement.
> Les priorités peuvent être ajustées selon les retours de la communauté et les tests en conditions réelles.
