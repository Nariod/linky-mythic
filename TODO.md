# linky-mythic — Migration Plan & Audit

> **For AI agents working on this project**: read this file in full before touching any code.
> Complete phases in order. Each task specifies exact files, diffs, and validation steps.
> Run the validation command at the end of each phase before moving on.

---

## Context and constraints

**Source project**: `linky/` — Rust C2 framework with a custom 3-stage HTTP protocol.
**Target project**: `linky-mythic/` — same Rust implants, adapted for the Mythic C2 framework.

**Single constraint**: HTTPS is the only supported C2 transport. The Mythic `http` C2 profile
handles TLS termination. No WebSocket, no SMB, no TCP — HTTP profile only.

**Do not touch**: `agent_capabilities.json`, `config.json`, `.gitignore`.
These files are already correct.

**Reference files** (read before writing any code):
- `agent_code/links/common/src/lib.rs` — Mythic protocol implementation
- `mythic/agent_functions/builder.go` — builder
- `mythic/agent_functions/shell.go` — canonical command definition example

**Écarts par rapport au plan initial** (déjà implémentés ainsi, ne pas changer) :
- Structure à la racine, pas sous `Payload_Type/linky/` — tous les chemins Go sont relatifs à la racine
- Constants CALLBACK/IMPLANT_SECRET/PAYLOAD_UUID dans `build.rs` → `env!()` dans `main.rs`
- `ureq 3` (remplace reqwest), `rand 0.10` (pas 0.8.5)
- `obfstr` utilisé pour les strings Mythic sensibles (checkin, get_tasking, etc.)

---

## Mythic wire format — mandatory reading

Every message between an implant and Mythic uses the **AES256_HMAC** scheme:

```
base64( UUID(36 bytes) + IV(16 bytes) + AES-256-CBC(PKCS7(JSON)) + HMAC-SHA256(32 bytes) )
```

- `UUID` (36 chars): during checkin, the `PAYLOAD_UUID` baked into the binary.
  After checkin succeeds, Mythic returns a `callback_id`; all subsequent messages use that.
- `IV` (16 bytes): random initialization vector for AES-CBC.
- `AES-256-CBC`: encryption with PKCS7 padding, block size 16.
- `HMAC-SHA256(32 bytes)`: computed over `IV + ciphertext` using the **same** AES key.
- The **entire message** (UUID + IV + ciphertext + HMAC) is base64-encoded.

The key is the raw 32-byte AESPSK from the HTTP C2 profile (base64-decoded at runtime).
It is provided to the builder as a base64-encoded string via `c2.GetCryptoArg("AESPSK")`.

---

## Completed phases

### Phase 0 — Repository layout ✅
### Phase 1 — Migrate Rust implant crates from Linky ✅
### Phase 2 — Clean up the Mythic wire format in `lib.rs` ✅
### Phase 3 — Complete the Go builder ✅
### Phase 4 — Expand command definitions in Go ✅
### Phase 5 — HTTPS configuration et Mythic HTTP profile ✅
### Phase 5b — Corriger le builder Go ✅
### Phase 5c — Unifier le dispatch Rust ✅
### Phase 5d — MVP fixes (Go migration + Rust quality) ✅
### Phase 6 — End-to-end testing contre Mythic ✅
### Phase 10 — Commandes manquantes (cp, mv, rm, mkdir, execute) ✅
### Phase 11 — Réduction taille binaires (reqwest→ureq, 4.5→1.9 MB) ✅
### Phase 12 — OPSEC hardening (obfstr + RUSTFLAGS) ✅

`go build ./...` + `cargo test --workspace` : 9/9 tests passent.
Build payload Linux via Mythic API : ✅ (54 MB debug build).
Prêt pour Phase 8 (hardening).

---

### Phase 5d — MVP fixes (Go migration + Rust quality) ✅

Critical issues discovered and fixed during MVP audit:

1. **Go migration** : `MythicContainerPkg` repo deleted (404). Migrated to
   `MythicContainer v1.6.4` (Go 1.25). All 16 command files, builder, payload_type,
   and main.go updated for new API (`[]string` OS types, `ParameterGroupInformation`,
   `CanBeWrappedByTheFollowingPayloadTypes`, `PayloadUUID` in build response).

2. **BUG-12** : Shell/inject/cmd/powershell dispatchers passed raw JSON to shell
   instead of extracting parameters. Fixed all 3 platform crates to use
   `extract_param(parameters, "command")` with fallback for backward compat.

3. **QUAL-03** : Removed `pub use base64; pub use serde_json;` re-exports from lib.rs.
4. **QUAL-04** : Replaced float math in `sleep_with_jitter` with integer-only arithmetic.
5. **QUAL-06** : `list_dir` now sorts results before joining.
6. **QUAL-07** : Removed all `.expect()` panics from `build_mythic_message` and
   `encrypt_config` — replaced with match-based error handling.
7. **Dockerfile** : Updated from `golang:1.21` to `golang:1.25`.
8. **CI workflow** : Updated `go-version` from `"1.21"` to `"1.25"`.

### Validation (Phase 5d)

```bash
go build ./... && go vet ./...
# OK — 0 errors

cd agent_code
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x CALLBACK_URI=/ cargo test --workspace
# 7 passed; 0 failed (3 common + 4 linux)
```

---

## Audit — Bugs critiques (résolus ✅)

Issues found during deep code review, April 2026. **All resolved as of 2026-04-03.**

### BUG-01 ✅ — `build_client()` : méthode inexistante + certs non acceptés

**Fichier** : `agent_code/links/common/src/lib.rs`, fonction `build_client()`

**Problème** : Le code utilise `.danger_accept_invalid_hostnames(true)` qui **n'existe pas**
dans reqwest 0.13. La bonne méthode est `.danger_accept_invalid_certs(true)`.
De plus, il manque un timeout et un User-Agent.

**Impact** : L'implant ne pourra pas se connecter au serveur Mythic (certificat self-signed rejeté).
Crash immédiat au premier POST.

**Fix** :
```rust
pub fn build_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client init failed")
}
```

---

### BUG-02 ✅ — Double préfixe `https://` dans l'URL de callback

**Fichier** : `agent_code/links/common/src/lib.rs`, fonction `run_c2_loop()`

**Problème** : Le code fait `format!("https://{}", decrypted_callback)` mais le C2 profile
Mythic retourne déjà `callback_host` avec le schéma inclus (`https://10.0.0.1`).
Le Go `builder.go` concatène `host:port`, ce qui donne un callback du genre
`https://10.0.0.1:443`. L'implant produit alors `https://https://10.0.0.1:443/`.

**Impact** : URL invalide, toutes les requêtes HTTP échouent.

**Fix** : Dans `run_c2_loop`, ne pas ajouter de schéma :
```rust
let base = if decrypted_callback.starts_with("http") {
    decrypted_callback.to_string()
} else {
    format!("https://{}", decrypted_callback)
};
```
**OU** dans `builder.go`, strip le schéma avant concaténation :
```go
host = strings.TrimPrefix(host.(string), "https://")
host = strings.TrimPrefix(host.(string), "http://")
callbackHost = fmt.Sprintf("%s:%s", host, port)
```

---

### BUG-03 ✅ — `PayloadUUID` : type string, pas `uuid.UUID`

**Fichier** : `mythic/agent_functions/builder.go`

**Problème** : `PayloadBuildMessage.PayloadUUID` est un `string` dans MythicContainerPkg,
pas un `uuid.UUID`. Le code fait `input.PayloadUUID.String()` (erreur de compilation :
string n'a pas de méthode `.String()`) et `input.PayloadUUID[:]` (retourne un string, pas
un `[]byte` — `hex.EncodeToString` ne compile pas).

**Impact** : Le builder Go **ne compile pas**.

**Fix** :
```go
import "github.com/google/uuid"

// Parse le string UUID pour obtenir les 16 bytes
parsedUUID, err := uuid.Parse(input.PayloadUUID)
if err != nil {
    resp.BuildStdErr = fmt.Sprintf("invalid PayloadUUID: %v", err)
    return resp
}
payloadUUID := input.PayloadUUID           // string "xxxxxxxx-xxxx-..."
aesKey := hex.EncodeToString(parsedUUID[:]) // 32-char hex from 16 bytes
```

---

### BUG-04 ✅ — Mismatch paramètres Go ↔ Rust pour `sleep` et `inject`

**Fichier Go** : `mythic/agent_functions/sleep.go`
**Fichier Rust** : `agent_code/links/linux/src/stdlib.rs`

**Problème** : Le Go `sleep.go` envoie un paramètre unique `args` (string `"30 10"`).
Mythic transmet au Rust `{"args": "30 10"}`. Mais le Rust fait
`extract_param(parameters, "seconds")` qui cherche la clé `"seconds"` → ne la trouve pas
→ retourne le JSON brut `{"args": "30 10"}` → échec du parse en u64.

Même problème pour `inject.go` qui envoie `args` mais le Rust attend
`extract_param(parameters, "pid")`.

**Impact** : `sleep` et `inject` ne fonctionnent jamais.

**Fix** — deux options :
1. **Option A** (recommandée) : aligner le Go sur des paramètres structurés :
```go
// sleep.go — remplacer le paramètre unique "args" par :
CommandParameters: []agentstructs.CommandParameter{
    {Name: "seconds", CLIName: "seconds", ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_NUMBER, Required: true},
    {Name: "jitter", CLIName: "jitter", ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_NUMBER, Required: false, DefaultValue: 0},
},
```
2. **Option B** : aligner le Rust sur `extract_param(parameters, "args")`.

---

### BUG-05 ✅ (faux positif) — `reqwest` feature `"rustls"` vs `"rustls-tls"`

**Fichier** : `agent_code/links/common/Cargo.toml`

**Analyse** : En reqwest 0.13, le feature s'appelle `"rustls"` (pas `"rustls-tls"` qui
existait en 0.11/0.12). Le feature `"rustls-tls"` n'existe tout simplement pas en 0.13.
De plus, `danger_accept_invalid_certs(true)` est en place dans `build_client()` — la
validation de certificats est bypassed, la question des root CAs est sans objet.

**Statut** : Aucun changement nécessaire. `"rustls"` est la bonne valeur en reqwest 0.13.

---

### BUG-06 ✅ — Crypto incompatible avec Mythic (AES-GCM → AES-256-CBC + HMAC-SHA256)

**Fichiers** : `agent_code/links/common/src/lib.rs`, `mythic/agent_functions/builder.go`, `agent_code/links/common/Cargo.toml`

**Problème** : L'agent utilisait AES-256-GCM avec une clé dérivée par `SHA-256(hex(UUID) + "mythic-salt")`.
Mythic attend **AES-256-CBC + HMAC-SHA256** avec la clé brute AESPSK du C2 profile.
Format Mythic : `base64( UUID(36) + IV(16) + AES-256-CBC(PKCS7(JSON)) + HMAC-SHA256(32) )`.
HMAC calculé sur `IV + ciphertext` avec la **même** clé AES.

**Impact** : Aucun callback live ne pouvait fonctionner — format crypto totalement incompatible.

**Fix** :
- **Rust** : Remplacé `aes-gcm` par `aes 0.8` + `cbc 0.1` + `hmac 0.12` + `sha2 0.10`.
  Supprimé `derive_key()`, ajouté `decode_aes_key()` (base64 → `[u8; 32]`).
  Réécrit `build_mythic_message()`, `parse_mythic_message()`, `encrypt_config()`, `decrypt_config()`.
- **Go** : Supprimé `uuid.Parse` + `hex.EncodeToString` pour la dérivation de clé.
  Ajouté `c2.GetCryptoArg("AESPSK")` pour récupérer la clé du C2 profile.
  Réécrit `encryptCallback()` en AES-256-CBC + HMAC-SHA256.
  Supprimé la dépendance `github.com/google/uuid`.
- **Cargo.toml** : `aes-gcm = "0.10"` → `aes = "0.8"`, `cbc = "0.1"`, `hmac = "0.12"`;
  `sha2 = "0.11"` → `sha2 = "0.10"` (compatibilité hmac); ajouté `zeroize = "1"`.

---

### BUG-07 ✅ — `cargo build --profile dev` → mauvais chemin de sortie

**Fichier** : `mythic/agent_functions/builder.go`

**Problème** : Quand `debug = true`, le code met `profile = "dev"` puis cherche le binaire
dans `target/<target>/dev/`. Mais cargo met les builds debug dans `target/<target>/debug/`,
pas `target/<target>/dev/`.

**Impact** : Build debug = "file not found" systématique.

**Fix** :
```go
outputProfile := profile
if profile == "dev" {
    outputProfile = "debug"
}
binaryPath := filepath.Join(crateDir, "target", target, outputProfile, binName+outputExt)
```

---

## Audit — Bugs importants (résolus ✅)

### BUG-08 ✅ — Architecture dispatch incohérente entre plates-formes

**Fichiers** :
- `agent_code/links/linux/src/stdlib.rs` — ne passe PAS par `dispatch_common`
- `agent_code/links/windows/src/stdlib.rs` — passe par `dispatch_common`
- `agent_code/links/osx/src/stdlib.rs` — passe par `dispatch_common`

**Problème** : Linux gère tous les cas directement avec `extract_param` (format JSON).
Windows/OSX passent par `dispatch_common` qui reconstitue une string `"{cmd} {params}"`
et la re-parse via `split_first`. Ce chemin perd la structure JSON des paramètres Mythic.

Conséquence : sur Linux `sleep {"seconds": 30, "jitter": 10}` fonctionne (via extract_param),
sur Windows/OSX la même commande passe par `dispatch_common` → `split_first("sleep {\"seconds\": 30}")` → args = `"{\"seconds\": 30}"` → `handle_sleep_command` tente `parse::<u64>` → échec.

**Fix** : Aligner les trois plates-formes sur le même modèle (celui de Linux).
Supprimer les appels à `dispatch_common` dans Windows/OSX et router directement
avec `extract_param`. Ou refactorer `dispatch_common` pour accepter `(command, parameters)`.

---

### BUG-09 ✅ — `download_file` format incompatible Mythic

**Fichier** : `agent_code/links/common/src/lib.rs`

**Problème** : `download_file` retournait `"FILE:path:base64_content"` — un format custom Linky.
Mythic attend un protocole multi-étapes : registration (total_chunks, full_path) → file_id → chunks.

**Impact** : Le contenu du fichier s'affichait en base64 brut dans l'UI Mythic au lieu
d'être téléchargeable.

**Fix** : Implémenté `mythic_download()` — protocole Mythic chunked file transfer complet :
1. Envoie `post_response` avec `download.total_chunks` et `download.full_path`
2. Reçoit `file_id` de Mythic
3. Envoie les chunks un par un avec `chunk_num`, `file_id`, `chunk_data` (base64)
Le download est géré directement dans `run_c2_loop` (pas via `dispatch_common`) car il
nécessite plusieurs aller-retours HTTP.

---

### BUG-10 ✅ — Commande `exit` non enregistrée côté Go

**Problème** : Le Rust gère `"exit"` dans `run_c2_loop` mais aucun fichier Go ne définit
cette commande → impossible de l'émettre depuis l'UI Mythic.

**Fix** : Créer `mythic/agent_functions/exit.go` :
```go
func registerExit() {
    agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
        Name: "exit", Description: "Terminate the implant", HelpString: "exit", Version: 1,
        CommandAttributes: agentstructs.CommandAttribute{
            SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS},
        },
        TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
            return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
        },
    })
}
```
Ajouter `registerExit()` à `RegisterAllCommands()` dans `builder.go`.

---

### BUG-11 ✅ — Pas de status `"error"` dans `TaskResponse`

**Fichier** : `agent_code/links/common/src/lib.rs`, boucle de polling

**Problème** : `TaskResponse.status` est toujours `None`. Mythic attend `"error"` quand
une commande échoue. Sans ce signal, l'UI marque tout comme "completed" même en cas d'erreur.

**Fix** :
```rust
let output = dispatch(&task.command, &task.parameters);
let is_error = output.starts_with("[-]");
responses.push(TaskResponse {
    task_id: task.id.clone(),
    completed: true,
    user_output: output,
    status: if is_error { Some("error".to_string()) } else { None },
});
```

---

### BUG-12 ✅ — Shell/inject dispatch passe le JSON brut au shell

**Fichiers** : `agent_code/links/{linux,windows,osx}/src/stdlib.rs`

**Problème** : Mythic envoie les paramètres de tâche en JSON structuré, par ex.
`{"command": "ls -la"}` pour shell ou `{"pid": 1234, "shellcode": "..."}` pour inject.
Les dispatchers platform passaient la string JSON brute directement à `shell_exec()` /
`inject_cmd()`, causant l'exécution de `sh -c '{"command": "ls -la"}'` au lieu de
`sh -c 'ls -la'`.

**Impact** : Toutes les commandes shell/cmd/powershell/inject échouent systématiquement
sur les 3 plates-formes.

**Fix** : Utiliser `link_common::extract_param(parameters, "command")` (ou `"pid"` / `"shellcode"`)
avant d'invoquer les fonctions d'exécution. Fallback sur le paramètre brut si l'extraction
retourne vide (compatibilité arrière avec le format texte simple).

---

## Audit — Code quality / Rust idiomatique

### QUAL-01 ✅ — `obfstr` et `zeroize` : dépendances corrigées

- `obfstr = "0.4"` retiré des trois crates platform (inutilisé, D1 deferred).
- `zeroize = "1"` ajouté à common/Cargo.toml et utilisé : `encryption_key.zeroize()`
  appelé en fin de `run_c2_loop` (sortie normale et commande `exit`).

---

### QUAL-02 ✅ — `derive_key` : copie manuelle inutile

```rust
// Actuel :
let result = h.finalize();
let mut key = [0u8; 32];
key.copy_from_slice(&result[..32]);
key

// Idiomatique :
h.finalize().into()
```

---

### QUAL-03 ✅ — `pub use base64; pub use serde_json;` anti-pattern

Re-exporter des dépendances entières au niveau module est fragile (breaking change upstream
= breaking change pour les consommateurs). Mieux : chaque crate platform dépend directement
de `base64` et `serde_json` dans son propre Cargo.toml.

**Fix** : Supprimé les deux `pub use` de `lib.rs`. Aucun consommateur n'utilisait ces re-exports.

---

### QUAL-04 ✅ — `sleep_with_jitter` : imprécision float et edge cases

```rust
let range = (base as f64 * jitter_pct as f64 / 100.0) as i64;
```
- Quand `base` > 2^53, la conversion `u64 → f64` perd de la précision.
- Quand `base = 0`, `range = 0`, `modulo = 1`, le sleep final est 1s (pas 0).
- Remplacement recommandé par du calcul entier pur.

**Fix** : Remplacé par `base * jitter_pct as u64 / 100` avec `saturating_sub/add`. Le cas `base = 0` retourne immédiatement via `sleep(0)`.

---

### QUAL-05 ✅ — `extract_param` fallback fragile

Quand le paramètre est du JSON valide mais que la clé n'existe pas, la fonction retourne
le JSON brut entier. Ça provoque des erreurs silencieuses en aval (le JSON brut est
passé comme chemin de fichier, comme durée de sleep, etc.).

**Fix** : retourner `""` quand la clé est absente :
```rust
pub fn extract_param(parameters: &str, key: &str) -> String {
    serde_json::from_str::<serde_json::Value>(parameters)
        .ok()
        .and_then(|v| {
            let val = v.get(key)?;
            Some(match val {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => val.to_string(),
            })
        })
        .unwrap_or_default() // "" au lieu du JSON brut
}
```
⚠️ Vérifier que tous les call sites gèrent le cas `""` (ex: `ls ""` → `ls "."` par défaut).

---

### QUAL-06 ✅ — `list_dir` : résultat non trié

La liste du répertoire est dans l'ordre du filesystem (aléatoire sur ext4).

**Fix** : Ajouté `.sort()` avant le `.join("\n")`.

---

### QUAL-07 ✅ — Panics dans le code crypto

`build_mythic_message` utilise `.expect("encrypt")` — un panic dans un implant en prod
= process terminé, callback perdu. Préférer un `return` avec message d'erreur.

**Fix** : Remplacé `.expect()` par `match` dans `build_mythic_message` et `encrypt_config`.
Retourne un string vide / la valeur en clair en cas d'échec crypto.

---

### QUAL-08 ✅ — `go.mod` : package name mismatch potentiel

Le `go.mod` importe `MythicContainerPkg` (ancien nom). Le package actuel est
`MythicContainer` (voir https://github.com/MythicMeta/MythicContainer).

**Fix** : Migré vers `MythicContainer v1.6.4` (Go 1.25). Tous les imports et usages API
mis à jour dans les 18 fichiers Go (main.go, payload_type.go, builder.go, 15 command files).

---

## Phase 5 — HTTPS configuration et Mythic HTTP profile ✅

Toutes les tâches complètes : BUG-01, BUG-02, BUG-05 (faux positif), CALLBACK_URI.

---

## Phase 5b — Corriger le builder Go ✅

Toutes les tâches complètes : BUG-03, BUG-07, BUG-04, BUG-10, QUAL-08.

---

## Phase 5c — Unifier le dispatch Rust ✅

Toutes les tâches complètes : BUG-08, BUG-11, QUAL-01, QUAL-02, QUAL-05.

### Validation (passée)

```bash
cd agent_code/links
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x CALLBACK_URI=/ cargo check --workspace
# Finished `dev` profile — 0 errors
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x CALLBACK_URI=/ cargo test --workspace
# 4 passed; 0 failed
```

---

## Phase 6 — End-to-end testing contre Mythic ✅

Validé le 2026-04-03 contre Mythic 3.4 sur Fedora (Docker rootless + SELinux).
**Live callback test réalisé le 2026-07-15** — implant Linux exécuté, checkin vérifié,
toutes les commandes testées.

### 6.1 — Set up local Mythic instance ✅

Mythic déployé dans `~/Documents/Mythic`. Tous les services migrés vers Docker named
volumes (`*_USE_VOLUME="true"`) pour contourner les problèmes SELinux/Docker rootless
avec les bind mounts.

```bash
# Dans ~/Documents/Mythic/.env — réglages nécessaires sur Fedora (Docker rootless)
POSTGRES_USE_VOLUME="true"
RABBITMQ_USE_VOLUME="true"
MYTHIC_SERVER_USE_VOLUME="true"
JUPYTER_USE_VOLUME="true"
DOCUMENTATION_USE_VOLUME="true"
HASURA_USE_VOLUME="true"
MYTHIC_REACT_USE_VOLUME="true"
NGINX_USE_VOLUME="true"
HTTP_USE_VOLUME="true"
```

### 6.2 — Bugs découverts et corrigés pendant l'intégration

#### BUG-13 ✅ — `agentDir` hardcodé à `/Mythic/agent_code`

**Fichier** : `mythic/agent_functions/builder.go`

**Problème** : Le chemin `agentDir` est hardcodé à `/Mythic/agent_code`, ce qui empêche
l'exécution du builder en dehors du container Docker (développement local, CI, etc.).

**Fix** : Ajout de la variable d'environnement `AGENT_CODE_DIR` avec fallback :
```go
agentDir := os.Getenv("AGENT_CODE_DIR")
if agentDir == "" {
    agentDir = "/Mythic/agent_code"
}
```

#### BUG-14 ✅ — Chemin du binaire compilé incorrect (workspace Cargo)

**Fichier** : `mythic/agent_functions/builder.go`

**Problème** : Le builder cherchait le binaire dans `crateDir/target/<target>/<profile>/`
(ex: `links/linux/target/...`), mais dans un workspace Cargo, le répertoire `target/`
est à la racine du workspace (`agentDir/target/...`).

**Impact** : Tous les builds échouent avec "no such file or directory" même quand cargo compile sans erreur.

**Fix** :
```go
// Avant (incorrect)
binaryPath := filepath.Join(crateDir, "target", target, outputProfile, binName+outputExt)
// Après (correct)
binaryPath := filepath.Join(agentDir, "target", target, outputProfile, binName+outputExt)
```

### 6.3 — Résultats du test d'intégration ✅

| Étape | Statut | Notes |
|-------|--------|-------|
| Connexion RabbitMQ | ✅ | Via env vars `RABBITMQ_HOST` / `RABBITMQ_PASSWORD` |
| Sync payload type "linky" | ✅ | `Successfully synced payload type!` |
| HTTP C2 profile | ✅ | Installé et `container_running: true` |
| Build payload Linux (debug) | ✅ | 55 MB, cargo build + binary returned |
| Build payload Linux (release) | ✅ | 4.5 MB, stripped, LTO, opt-level=z |
| Build payload Windows (release) | ✅ | Cross-compile mingw-w64 |
| Build payload macOS (release) | ❌ | `aws-lc-sys` needs osxcross + cmake |
| Go build | ✅ | `go build ./...` |
| Rust tests (9/9) | ✅ | 5 common + 4 linux |
| Live callback Linux | ✅ | Implant checkin via HTTP C2 profile (HTTPS port 443) |

### 6.4 — Matrice de test des commandes (live — juillet 2026)

> Testé avec un implant Linux release exécuté localement, callback via HTTP C2 profile
> avec SSL sur port 443. Toutes les commandes ont été émises depuis l'API Mythic GraphQL.

| Command | Input | Résultat | Statut |
|---------|-------|----------|--------|
| whoami | - | `fedora@<hostname>` | ✅ (hostname fix appliqué) |
| pwd | - | `/home/fedora/Documents/linky-mythic` | ✅ |
| ls | - | listing trié du répertoire courant | ✅ |
| ls /tmp | `{"path":"/tmp"}` | listing de /tmp | ✅ |
| cd /tmp | `{"path":"/tmp"}` | `[+] /tmp` | ✅ (fix appliqué — était vide) |
| pid | - | PID du process | ✅ |
| info | - | OS, arch, user, hostname, IP | ✅ |
| ps | - | liste des processus | ✅ |
| netstat | - | connexions réseau | ✅ |
| shell id | `id` | `uid=1000(fedora) gid=1000(fedora) ...` | ✅ |
| download | `/etc/os-release` | contenu fichier téléchargé | ✅ |
| killdate | `{"date":"9999999999"}` | confirmation date modifiée | ✅ |
| sleep | `30 10` | ❌ BUG-15 (fix appliqué, à retester) |
| upload | `{"file": ..., "remote_path": ...}` | ❌ BUG-19 (nécessite upload modal Mythic) |
| exit | - | ⬜ non testé (implant tué manuellement) |

### 6.5 — Crypto cross-test Go ↔ Rust ✅

Vérifié live : le builder Go chiffre la callback avec AES-256-CBC + HMAC-SHA256,
l'implant Rust déchiffre la configuration au démarrage, le checkin et le polling
fonctionnent correctement. Le crypto est 100% compatible Go ↔ Rust ↔ Mythic.

### 6.6 — Taille des binaires

| Build | Taille | Notes |
|-------|--------|-------|
| Linux debug | 55 MB | ELF 64-bit, static-pie, non strippé |
| Linux release | 4.5 MB | stripped, LTO, opt-level=z, panic=abort |
| Windows release | ~5 MB | PE, mingw-w64, stripped |
| macOS release | N/A | build échoue (osxcross requis) |

Principal contributeur : `reqwest` tire tokio, hyper, rustls (via aws-lc-sys).
Voir Phase 11 pour le plan de réduction.

---

## Bugs découverts pendant le test live (juillet 2026)

### BUG-15 ✅ — `sleep` : Mythic ne peut pas parser les paramètres string

**Fichier** : `mythic/agent_functions/sleep.go`

**Problème** : `sleep.go` ne définissait pas de `TaskFunctionParseArgString` handler.
Quand l'opérateur tape `sleep 30 10` dans la CLI Mythic, le framework ne sait pas
mapper la chaîne `"30 10"` vers les paramètres `seconds` et `jitter`.

**Impact** : La commande sleep échoue systématiquement côté Mythic (avant même l'envoi
au callback).

**Fix** : Ajouté `TaskFunctionParseArgString` qui split la string avec `strings.Fields`
et mappe `parts[0]` → `seconds`, `parts[1]` → `jitter`.

---

### BUG-16 ✅ — `cd` retourne une réponse vide

**Fichier** : `agent_code/links/common/src/dispatch.rs`

**Problème** : La commande `cd` réussissait mais retournait une string vide dans l'UI Mythic.
L'opérateur ne voyait aucune confirmation que le répertoire avait changé.

**Fix** : `cd` retourne maintenant `[+] /nouveau/chemin` en cas de succès.

---

### BUG-17 ✅ — `whoami` affiche un hostname vide

**Fichier** : `agent_code/links/linux/src/stdlib.rs`

**Problème** : `hostname()` lisait uniquement `/etc/hostname`, qui n'existe pas sur
certaines distributions (notamment Fedora avec systemd-hostnamed). Résultat :
`fedora@` au lieu de `fedora@myhostname`.

**Fix** : Fallback sur `/proc/sys/kernel/hostname` quand `/etc/hostname` n'existe pas.

---

### BUG-18 ⬜ — Builder strip le schéma HTTP — pas de support HTTP plain

**Fichier** : `mythic/agent_functions/builder.go` (lignes 66-68)

**Problème** : Le builder Go strip systématiquement `https://` et `http://` du callback_host.
L'implant Rust reconstruit l'URL en préfixant `https://` si absent. Conséquence : il est
**impossible** d'utiliser un C2 profile HTTP non-TLS (plain HTTP).

**Impact** : Opérationnel uniquement en HTTPS. Un C2 profile HTTP sur port 80 ne fonctionnera pas.

**Fix proposé** : Préserver le schéma dans la callback chiffrée, ou ajouter un paramètre de build
`use_ssl` (boolean) qui contrôle le préfixe appliqué par l'implant.

---

### BUG-19 ⬜ — `upload` non testable en CLI (nécessite modal Mythic)

**Fichier** : `mythic/agent_functions/upload.go`

**Problème** : La commande `upload` utilise `COMMAND_PARAMETER_TYPE_FILE` qui nécessite
un upload via le modal Mythic dans l'UI web. Impossible de tester via l'API GraphQL
sans passer par `mythic_utilities.SendFileToMythic()` au préalable.

**Impact** : Non bloquant — fonctionne probablement via l'UI web, mais non vérifié.

---

### OPSEC-01 ⬜ — Strings en clair dans le binaire

**Fichier** : Binaire compilé (toutes plates-formes)

**Problème** : L'analyse avec `strings` du binaire Linux release révèle :
- Noms d'actions Mythic en clair : `"checkin"`, `"get_tasking"`, `"post_response"`
- Messages d'erreur descriptifs : `"[-] failed to read file"`, etc.
- Chemins cargo/registry : `/root/.cargo/registry/src/...`
- User-Agent en clair : `"Mozilla/5.0 (Windows NT 10.0; ..."`
- Noms de crates : `reqwest`, `hyper`, `rustls`

**Impact** : Facilite la détection et le reverse engineering. Les EDR peuvent signer
ces strings comme IOC.

**Fix** : Voir Phase 12 (OPSEC hardening) — `obfstr!()`, strip des symboles debug,
compilation avec `RUSTFLAGS="-C debuginfo=0"`.

---

## Phase 7 — Upload et Download via Mythic file store ✅

### 7.1 — Download natif Mythic ✅

Implémenté `mythic_download()` dans `lib.rs` — protocole Mythic chunked file transfer :
1. Registration : `post_response` avec `download { total_chunks, full_path, chunk_size }`
2. Mythic retourne `file_id`
3. Agent envoie les chunks : `download { chunk_num, file_id, chunk_data }`
4. Chunk size : 512 KB

### 7.2 — Upload natif Mythic ✅

Implémenté `mythic_upload()` dans `lib.rs` — protocole Mythic pull-down :
1. Agent reçoit `file_id` + `remote_path` dans les task parameters
2. Agent envoie `upload { chunk_size, file_id, chunk_num, full_path }` à Mythic
3. Mythic retourne `chunk_data` (base64) + `total_chunks`
4. Agent itère sur tous les chunks et écrit le fichier

Go `upload.go` mis à jour avec les paramètres corrects :
- `file` (COMMAND_PARAMETER_TYPE_FILE) — le fichier sélectionné par l'opérateur
- `remote_path` (STRING) — le chemin de destination sur la cible

### 7.3 — Adapter le dispatch Rust ✅

`download` et `upload` sont gérés directement dans `run_c2_loop` (pas via `dispatch_common`)
car ils nécessitent plusieurs aller-retours HTTP avec le serveur Mythic.
`dispatch_common` ne contient plus ces commandes.

---

## Phase 8 — Hardening et OPSEC (partiellement ✅)

### 8.1 — `obfstr!()` sur les strings sensibles (D1) ⬜

Wrapper `"checkin"`, `"get_tasking"`, `"post_response"`, action strings avec `s!()`.

### 8.2 — Vrai secret AES (D2) ✅

La clé AES est maintenant le AESPSK du C2 profile — un secret 32 bytes aléatoire généré
par Mythic, totalement indépendant du PayloadUUID. Récupéré via `c2.GetCryptoArg("AESPSK")`.

### 8.3 — Sleep jitter sur le retry checkin (D3) ✅

Ajouté `sleep_with_jitter(retry_delay, 30)` au retry loop du checkin avec backoff exponentiel.

### 8.4 — Zeroize les clés en mémoire (QUAL-01) ✅

`encryption_key.zeroize()` appelé à la fin de `run_c2_loop` (sortie normale et commande `exit`).
Crate `zeroize = "1"` ajouté à `Cargo.toml`.

### 8.5 — Éliminer les panics dans le code crypto (QUAL-07) ✅

Déjà résolu en Phase 5d.

---

## Phase 9 — Documentation et CI ⬜

### 9.1 — Process browser Mythic

Implémenter le callback `process_browser` pour `ps` (format JSON Mythic).

### 9.2 — File browser Mythic

Implémenter le callback `file_browser` pour `ls` (format JSON Mythic).

### 9.3 — Hugo documentation

### 9.4 — CI pipeline fonctionnel

Les scripts `setup_test_env.sh`, `run_tests.sh`, `test_integration.sh` sont des stubs.
Les remplacer par une CI réelle avec Docker-in-Docker et Mythic.

---

## Known issues and deferred items

| ID | Description | Phase |
|----|-------------|-------|
| D1 | `obfstr!()` sur les strings d'action Mythic | Phase 12 |
| D2 | ~~IMPLANT_SECRET = hex(UUID) est faible~~ | ✅ Done — AESPSK 32 bytes aléatoire |
| D3 | ~~Pas de jitter sur le retry checkin~~ | ✅ Done — backoff exponentiel + jitter |
| D4 | macOS cross-compilation nécessite osxcross (non inclus dans le Dockerfile) | Phase 13 |
| D5 | URI hardcodé à `"/`" | ✅ Done (Phase 5) |
| D6 | Pre-commit hook Rust workspace validation | ✅ Done |
| D7 | reqwest version compatibility | ✅ Done — `"rustls"` correct en reqwest 0.13 |
| D8 | Test scripts are stubs | Phase 9 |
| D9 | ~~Callback live test~~ | ✅ Done — testé live avec implant Linux |
| D10 | ~~Vérifier `MythicEncryptsData` (BUG-06)~~ | ✅ Done — crypto Mythic-compatible |
| D11 | Builder strip le schéma HTTP (BUG-18) — HTTPS uniquement | Phase 10 |
| D12 | Upload non testable via API (BUG-19) | Phase 9 |
| D13 | Strings OPSEC en clair dans les binaires (OPSEC-01) | Phase 12 |

---

## Comparatif Hannibal vs linky-mythic

> Référence : [silentwarble/Hannibal](https://github.com/silentwarble/Hannibal) — agent C2 Mythic
> écrit en C, Windows uniquement, 25-45 KB, avec sleep obfuscation et HBIN.

### Résumé des forces

| Aspect | Hannibal | linky-mythic | Avantage |
|--------|----------|-------------|----------|
| Taille binaire | 25-45 KB | 4.5 MB | **Hannibal** |
| Sleep obfuscation | Ekko (RC4 .text encryption) | Aucune | **Hannibal** |
| String obfuscation | Hash compile-time (ROL5) | Aucune | **Hannibal** |
| Post-exploitation | HBIN modules chargés dynamiquement | Aucune | **Hannibal** |
| PEB walking / API hashing | Oui (bypass IAT hooks) | Non | **Hannibal** |
| Cross-plateforme | Windows x64 uniquement | Linux + Windows + macOS | **linky-mythic** |
| Safety mémoire | C (risque de UB) | Rust (garanti à la compilation) | **linky-mythic** |
| Maintenabilité | Faible (PIC C, linker scripts) | Haute (Rust idiomatique, workspace) | **linky-mythic** |
| Vitesse de build | ~2 minutes | ~30 secondes | **linky-mythic** |
| Tests unitaires | Aucun | 9 tests (Go + Rust) | **linky-mythic** |
| Audit trail | Non | Oui (via Mythic) | **linky-mythic** |

### Commandes exclusives à Hannibal (à intégrer)

| Commande | Description | Priorité |
|----------|-------------|----------|
| `cp` | Copie récursive fichier/dossier | P1 — Phase 10 |
| `mv` | Déplacer/renommer fichier | P1 — Phase 10 |
| `rm` | Suppression récursive | P1 — Phase 10 |
| `mkdir` | Création répertoire | P1 — Phase 10 |
| `execute` | Exécution directe de binaire (pas via shell) | P2 — Phase 10 |
| `execute_hbin` | Chargement dynamique de modules HBIN | P3 — Phase 14 |
| `ipinfo` | Informations interfaces réseau | P2 — Phase 10 |
| `listdrives` | Disques montés (Windows) | P2 — Phase 10 |
| `hostname` | Hostname seul (déjà dans `info`) | Déjà couvert |

### Techniques exclusives à Hannibal (à rechercher)

| Technique | Description | Applicable en Rust ? |
|-----------|-------------|---------------------|
| Ekko sleep | Chiffrement RC4 de .text pendant sleep | Partiel — nécessite Win32 APIs (Windows only) |
| PEB walking | Résolution manuelle DLL/API via PEB | Oui — via `ntapi` crate ou inline asm |
| API hashing | Hash compile-time des noms d'API | Oui — via `obfstr` + macros Rust |
| Custom linker script | Sections minimales, pas de .pdata | Partiel — Rust supporte les linker scripts custom |
| nostdlib | Pas de CRT, implémentations custom | Difficile — Rust std est deeply intégré |
| Shellcode extraction | Extraction .text seul depuis PE | Oui — profile `release-shellcode` existe déjà |
| Modular DLL loading | Chargement conditionnel des DLLs | Oui — via `cfg` features Cargo |
| Modular commands | `#define INCLUDE_CMD_*` | Oui — via `cfg` features Cargo |

---

## Phase 10 — Commandes manquantes (parité Hannibal) ✅

Implémenté : `cp`, `mv`, `rm`, `mkdir`, `execute` (cross-platform dans dispatch.rs).
Go command definitions créées. BUG-18 (schéma HTTP) corrigé dans builder.go.
Testé sur Mythic live — toutes les commandes fonctionnent.

### 10.1 — File operations (cross-platform)

Ajouter les commandes `cp`, `mv`, `rm`, `mkdir` à `dispatch.rs` (commun) :

```rust
"cp" => {
    let src = crate::extract_param(parameters, "source");
    let dst = crate::extract_param(parameters, "destination");
    match std::fs::copy(&src, &dst) {
        Ok(bytes) => format!("[+] copied {} bytes", bytes),
        Err(e) => format!("[-] {}", e),
    }
}
"mv" => {
    let src = crate::extract_param(parameters, "source");
    let dst = crate::extract_param(parameters, "destination");
    match std::fs::rename(&src, &dst) {
        Ok(_) => format!("[+] moved {} -> {}", src, dst),
        Err(e) => format!("[-] {}", e),
    }
}
"rm" => {
    let path = crate::extract_param(parameters, "path");
    let meta = std::fs::metadata(&path);
    let result = match meta {
        Ok(m) if m.is_dir() => std::fs::remove_dir_all(&path),
        _ => std::fs::remove_file(&path),
    };
    match result {
        Ok(_) => format!("[+] removed {}", path),
        Err(e) => format!("[-] {}", e),
    }
}
"mkdir" => {
    let path = crate::extract_param(parameters, "path");
    match std::fs::create_dir_all(&path) {
        Ok(_) => format!("[+] created {}", path),
        Err(e) => format!("[-] {}", e),
    }
}
```

Go command definitions : créer `cp.go`, `mv.go`, `rm.go`, `mkdir.go` avec les paramètres
`source`/`destination` ou `path`.

### 10.2 — `execute` (exécution directe sans shell)

Exécuter un binaire directement (sans passer par `/bin/sh -c`) :
```rust
"execute" => {
    let args_raw = crate::extract_param(parameters, "command");
    let parts: Vec<&str> = args_raw.split_whitespace().collect();
    if parts.is_empty() { return Some("[-] no command".into()); }
    match std::process::Command::new(parts[0]).args(&parts[1..]).output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string() +
                 &String::from_utf8_lossy(&o.stderr),
        Err(e) => format!("[-] {}", e),
    }
}
```

### 10.3 — `ipinfo` (informations réseau)

- **Linux** : parser `/proc/net/if_inet6` et `/sys/class/net/*/address`
- **Windows** : `GetAdaptersAddresses` via `windows-sys` crate
- **macOS** : `ifconfig` shell fallback

### 10.4 — `listdrives` (Windows only)

- `GetLogicalDriveStringsW` + `GetDiskFreeSpaceExW` via `windows-sys`

### 10.5 — Support protocole HTTP plain (BUG-18)

Préserver le schéma dans la callback chiffrée au lieu de le stripper dans builder.go.
L'implant utilise le schéma tel quel.

---

## Phase 11 — Réduction de la taille des binaires ✅ (partiel)

### Résultat

| Plateforme | Avant (reqwest) | Après (ureq) | Réduction |
|-----------|----------------|--------------|-----------|
| Linux release | 4.5 MB | **1.9 MB** | -58% |

### 11.1 ✅ — Remplacer `reqwest` par `ureq`

Migration complète vers `ureq 3.3` avec `rustls` (crypto provider: `ring`).
Élimine tokio, hyper, hyper-util. Build et tests passent.

### 11.2 ✅ — `ring` au lieu de `aws-lc-sys`

ureq 3 utilise `ring` par défaut — pas besoin de cmake pour la cross-compilation.
Résout également le problème de build macOS.

### 11.3 — Implémenter les crypto primitives sans crates externes

Pour une réduction maximale, remplacer `aes`, `cbc`, `hmac`, `sha2` par des
implémentations inline (type `RustCrypto` minimal ou implémentation directe).

**Risque** : Élevé — implémentations crypto custom = risque de vulnérabilités.
**Économie estimée** : 100-300 KB

### 11.4 — `cargo-bloat` analysis et élimination des dépendances

```bash
cargo install cargo-bloat
cargo bloat --release --target x86_64-unknown-linux-musl -n 30
```

Identifier les 30 plus gros symboles et évaluer s'ils sont nécessaires.

### 11.5 — UPX compression post-build (optionnel)

```bash
upx --best --lzma target/x86_64-unknown-linux-musl/release/link-linux
```

**Économie** : 60-70% de compression (4.5 MB → ~1.5 MB)
**Inconvénient** : détecté par certains AV comme packer suspect

### 11.6 — Compilation conditionnelle des commandes via Cargo features

Inspiré de Hannibal : chaque commande est une feature Cargo. Le builder active
uniquement les features sélectionnées par l'opérateur.

```toml
[features]
default = ["cmd-shell", "cmd-ls", "cmd-cd", "cmd-pwd", "cmd-download", "cmd-upload"]
cmd-shell = []
cmd-ls = []
cmd-cd = []
cmd-pwd = []
cmd-ps = []
cmd-netstat = []
cmd-download = []
cmd-upload = []
cmd-inject = []  # Windows only
```

```rust
#[cfg(feature = "cmd-ls")]
"ls" => { /* ... */ }
```

Le builder Go passe les features activées :
```go
features := getSelectedFeatures(input.BuildParameters)
args = append(args, "--features", strings.Join(features, ","))
```

---

## Phase 12 — OPSEC hardening ✅ (partiel)

### 12.1 ✅ — String obfuscation avec `obfstr`

Strings Mythic sensibles obfusquées : `checkin`, `get_tasking`, `post_response`,
`download`, `upload`, user-agent. Empêche l'identification statique du protocole.

### 12.2 ✅ — Supprimer les chemins cargo du binaire

RUSTFLAGS ajouté dans builder.go : `--remap-path-prefix` + `-C debuginfo=0`.

### 12.1 — String obfuscation avec `obfstr`

Wrapper toutes les strings sensibles avec `obfstr::obfstr!()` :

```rust
use obfstr::obfstr as s;

let action = s!("checkin");
let tasking = s!("get_tasking");
let post = s!("post_response");
let ua = s!("Mozilla/5.0 (Windows NT 10.0; Win64; x64)...");
```

### 12.2 — Supprimer les chemins cargo du binaire

```bash
RUSTFLAGS="-C debuginfo=0 --remap-path-prefix=$HOME/.cargo/registry=. --remap-path-prefix=$(pwd)=."
```

Ajouter ces flags dans `builder.go` :
```go
cmd.Env = append(cmd.Env, "RUSTFLAGS=-C debuginfo=0 --remap-path-prefix=...")
```

### 12.3 — User-Agent configurable

Actuellement hardcodé dans `build_client()`. Le rendre paramétrable via le C2 profile :
- Ajouter un build parameter `user_agent` dans `payload_type.go`
- Passer en variable d'environnement au build
- `build.rs` le lit, `main.rs` le stocke en constante

### 12.4 — Process browser Mythic (`ps` structuré)

Remplacer la sortie texte de `ps` par le format JSON Mythic `process_browser` :

```json
{
    "processes": [
        {
            "process_id": 1234,
            "name": "sshd",
            "ppid": 1,
            "user": "root",
            "command_line": "/usr/sbin/sshd -D"
        }
    ]
}
```

Nécessite d'implémenter le parsing `/proc/[pid]/` natif (Linux) au lieu de shell `ps aux`.

### 12.5 — File browser Mythic (`ls` structuré)

Retourner le format JSON Mythic `file_browser` au lieu de texte brut :

```json
{
    "host": "hostname",
    "is_file": false,
    "name": "Documents",
    "parent_path": "/home/user",
    "files": [
        {
            "name": "file.txt",
            "is_file": true,
            "size": 1234,
            "permissions": { "permissions": "rw-r--r--" }
        }
    ]
}
```

### 12.6 — Sleep obfuscation (Windows — recherche)

Techniques à évaluer pour Rust :
- **Ekko-style** : `CreateTimerQueueTimer` + `VirtualProtect` + RC4 — possible via `windows-sys`
- **Foliage** : APC-based sleep — possible
- **Zilean** : Similar to Ekko with different timer mechanism

Contrainte : Rust ne donne pas accès direct au layout de `.text` — nécessite soit
du code `unsafe` avec des pointeurs bruts, soit un loader shellcode qui chiffre la mémoire.

### 12.7 — Indirect syscalls (Windows)

Utiliser `ntapi` ou `syswhispers`-equivalent en Rust pour appeler NtAllocateVirtualMemory,
NtWriteVirtualMemory, NtCreateThreadEx directement au lieu des APIs Win32 hookées.

---

## Phase 13 — macOS support complet ⬜

### 13.1 — Intégrer osxcross dans le Dockerfile

```dockerfile
# osxcross installation
RUN git clone https://github.com/tpoechtrager/osxcross /opt/osxcross
COPY MacOSX*.sdk.tar.xz /opt/osxcross/tarballs/
RUN cd /opt/osxcross && UNATTENDED=1 ./build.sh
ENV PATH="/opt/osxcross/target/bin:${PATH}"
RUN rustup target add x86_64-apple-darwin aarch64-apple-darwin
```

### 13.2 — Résoudre la dépendance `aws-lc-sys` pour macOS

Option A : Passer à `ring` comme crypto backend rustls (Phase 11.2).
Option B : Configurer cmake + Apple SDK dans le Dockerfile.

### 13.3 — Ajouter le support ARM64 (aarch64)

```toml
# Targets supplémentaires
aarch64-unknown-linux-musl
aarch64-apple-darwin
```

---

## Phase 14 — Post-exploitation avancée ⬜

### 14.1 — Dynamic module loading (équivalent HBIN)

Concept : charger et exécuter du code Rust compilé dynamiquement (`.so` sur Linux,
`.dll` sur Windows) au lieu de modules C.

Approches :
1. **Shared library loading** : `dlopen` / `LoadLibrary` avec une interface FFI commune
2. **Shellcode execution** : allocation mémoire + exécution (déjà implémenté via `inject`)
3. **WASM** : modules WebAssembly exécutés dans un runtime embarqué (wasmtime-minimal)

### 14.2 — Credential access

- **Linux** : lecture `/etc/shadow` (requiert root), dump de keyring
- **Windows** : intégration Mimikatz via HBIN/BOF, ou port Rust partiel

### 14.3 — Keylogging

- **Linux** : lecture `/dev/input/event*` (requiert root)
- **Windows** : `SetWindowsHookExW` via `windows-sys`

### 14.4 — SOCKS proxy

Implémentation d'un proxy SOCKS5 interne pour le pivoting réseau via Mythic.

---

## Phase 15 — CI/CD et qualité ⬜

### 15.1 — CI pipeline fonctionnel

Remplacer les stubs `setup_test_env.sh`, `run_tests.sh`, `test_integration.sh` par
une CI GitHub Actions complète :

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-unknown-linux-musl,x86_64-pc-windows-gnu
      - run: go build ./...
      - run: go vet ./...
      - run: |
          cd agent_code
          CALLBACK=x IMPLANT_SECRET=$(python3 -c "...") PAYLOAD_UUID=x CALLBACK_URI=/ \
            cargo test --workspace
      - run: |
          cd agent_code
          cargo build --release --target x86_64-unknown-linux-musl
          cargo build --release --target x86_64-pc-windows-gnu
      - run: |
          ls -la agent_code/target/*/release/link-*
```

### 15.2 — Tests d'intégration avec Mythic

Docker-in-Docker avec Mythic + HTTP C2 profile pour tester le pipeline build complet.

### 15.3 — Hugo documentation

Documentation Mythic-compatible pour l'agent (installation, usage, commandes).

---

## File checklist — état actuel (phases 0-8 complètes)

```
agent_code/
└── links/
    ├── common/
    │   ├── Cargo.toml                 ✅ aes+cbc+hmac+sha2+zeroize (AES-256-CBC+HMAC-SHA256)
    │   └── src/
    │       ├── lib.rs                 ✅ Mythic AES256_HMAC crypto, chunked file transfers, zeroize
    │       └── dispatch.rs            ✅ dispatch_common (download/upload gérés dans run_c2_loop)
    ├── linux/
    │   ├── build.rs                   ✅ CALLBACK_URI
    │   └── src/
    │       ├── main.rs                ✅ const CALLBACK_URI
    │       └── stdlib.rs              ✅ extract_param pour shell dispatch
    ├── windows/
    │   ├── build.rs                   ✅ CALLBACK_URI
    │   └── src/
    │       ├── main.rs                ✅ const CALLBACK_URI
    │       └── stdlib.rs              ✅ extract_param pour shell/cmd/powershell/inject
    └── osx/
        ├── build.rs                   ✅ CALLBACK_URI
        └── src/
            ├── main.rs                ✅ const CALLBACK_URI
            └── stdlib.rs              ✅ extract_param pour shell dispatch

(root)/
├── main.go                            ✅ MythicContainer import
├── go.mod                             ✅ MythicContainer v1.6.4, Go 1.25
├── Dockerfile                         ✅ golang:1.25
├── .github/workflows/test.yml         ✅ go-version: 1.25
└── mythic/
    ├── payload_type.go                ✅ []string OS, MythicEncryptsData: true
    └── agent_functions/
        ├── builder.go                 ✅ AESPSK via GetCryptoArg, AES-CBC+HMAC encryptCallback
        ├── shell.go                   ✅ ParameterGroupInformation
        ├── ls.go, cd.go, pwd.go ...   ✅ MythicContainer imports + OS constants
        ├── sleep.go                   ✅ ParameterGroupInformation
        ├── inject.go                  ✅ ParameterGroupInformation
        ├── exit.go                    ✅
        ├── download.go                ✅ path parameter
        └── upload.go                  ✅ file (FILE type) + remote_path parameters
```