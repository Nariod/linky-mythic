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
- `reqwest 0.13` (pas 0.12), `rand 0.10` (pas 0.8.5)
- `obfstr` retiré de `lib.rs` (strings JSON visibles sur le réseau de toute façon)

---

## Mythic wire format — mandatory reading

Every message between an implant and Mythic is:

```
<UUID_36_chars><base64(nonce_12_bytes || ciphertext)>
```

- `UUID` (36 chars): during checkin, the `PAYLOAD_UUID` baked into the binary.
  After checkin succeeds, Mythic returns a `callback_id`; all subsequent messages use that.
- `nonce` (12 bytes): random AES-GCM nonce, prepended to ciphertext — **raw bytes, not hex**.
- `ciphertext`: AES-256-GCM encrypted JSON payload.
- **The entire nonce+ciphertext blob is base64-encoded**, not hex-encoded.

The key is: `SHA-256(IMPLANT_SECRET || "mythic-salt")` — a 32-byte derived key.

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

`cargo check/test --workspace` : 4/4 tests passent. Prêt pour Phase 6.

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

### BUG-06 — `MythicEncryptsData` et double encryption potentielle

**Fichier** : `mythic/payload_type.go`

**Problème** : `MythicEncryptsData: true` indique à Mythic que le framework gère le chiffrement.
Si le C2 profile HTTP tente de déchiffrer les messages de l'agent AVANT de les transmettre
au core, et que l'agent fait son propre AES-GCM, il y aura double encryption.

**Action requise** : Vérifier contre une instance Mythic live. Si le C2 profile HTTP
déchiffre côté proxy, mettre `MythicEncryptsData: false` et laisser l'agent gérer
intégralement le chiffrement. Si Mythic Core déchiffre, garder `true` mais s'assurer
que le format `UUID + base64(nonce + AESGCM)` correspond exactement à ce que Mythic attend.

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

### BUG-09 — `download_file` format incompatible Mythic

**Fichier** : `agent_code/links/common/src/lib.rs`

**Problème** : `download_file` retourne `"FILE:path:base64_content"` — un format custom Linky.
Mythic attend un `post_response` structuré avec des champs spécifiques pour les file transfers
(file registration API, chunking, etc.).

**Impact** : Le contenu du fichier s'affiche en base64 brut dans l'UI Mythic au lieu
d'être téléchargeable.

**Fix** : Phase 7 (existante) couvre partiellement ce sujet. Il faut aussi adapter `download`:
```rust
"download" => {
    // Retourner le contenu comme user_output pour l'instant
    // Phase 7 : utiliser le Mythic file transfer API
    let path = extract_param(parameters, "path");
    match std::fs::read(&path) {
        Ok(buf) => {
            use base64::{engine::general_purpose::STANDARD, Engine as _};
            // Mythic format: full_path + total_chunks + chunk_num + chunk_data
            format!("FILE:{}:{}", path, STANDARD.encode(&buf))
        }
        Err(e) => format!("[-] {}", e),
    }
}
```

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

## Audit — Code quality / Rust idiomatique

### QUAL-01 ✅ — `obfstr` et `zeroize` : dépendances inutilisées

- `obfstr = "0.4"` est listé dans linux/windows/osx Cargo.toml mais jamais importé.
- `zeroize = "1.8"` est listé dans common/Cargo.toml mais jamais utilisé.
  La clé AES `[u8; 32]` n'est jamais zéroïsée après usage.

**Fix** : Retirer `obfstr` des trois crates (sauf si réintroduit en Phase D1).
Soit retirer `zeroize`, soit l'utiliser réellement (recommandé) :
```rust
use zeroize::Zeroize;
// À la fin de run_c2_loop :
encryption_key.zeroize();
```

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

### QUAL-03 — `pub use base64; pub use serde_json;` anti-pattern

Re-exporter des dépendances entières au niveau module est fragile (breaking change upstream
= breaking change pour les consommateurs). Mieux : chaque crate platform dépend directement
de `base64` et `serde_json` dans son propre Cargo.toml.

---

### QUAL-04 — `sleep_with_jitter` : imprécision float et edge cases

```rust
let range = (base as f64 * jitter_pct as f64 / 100.0) as i64;
```
- Quand `base` > 2^53, la conversion `u64 → f64` perd de la précision.
- Quand `base = 0`, `range = 0`, `modulo = 1`, le sleep final est 1s (pas 0).
- Remplacement recommandé par du calcul entier pur.

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

### QUAL-06 — `list_dir` : résultat non trié

La liste du répertoire est dans l'ordre du filesystem (aléatoire sur ext4).
Ajouter un `.sort()` avant le `.join("\n")`.

---

### QUAL-07 — Panics dans le code crypto

`build_mythic_message` utilise `.expect("encrypt")` — un panic dans un implant en prod
= process terminé, callback perdu. Préférer un `return` avec message d'erreur.

---

### QUAL-08 — `go.mod` : package name mismatch potentiel

Le `go.mod` importe `MythicContainerPkg` (ancien nom). Le package actuel est
`MythicContainer` (voir https://github.com/MythicMeta/MythicContainer).
Vérifier que v1.3.10 existe bien sous l'ancien nom, sinon migrer.

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

## Phase 6 — End-to-end testing contre Mythic

### 6.1 — Set up local Mythic instance

```bash
git clone https://github.com/its-a-feature/Mythic
cd Mythic && make
sudo ./mythic-cli start
sudo ./mythic-cli install github https://github.com/MythicC2Profiles/http
sudo ./mythic-cli install folder /path/to/linky-mythic
```

### 6.2 — Vérifier `MythicEncryptsData` (BUG-06)

Tester le checkin. Si double encryption détectée → changer à `false`.

### 6.3 — Checkin test

1. Générer un payload Linux dans l'UI Mythic
2. Exécuter le binaire
3. Vérifier qu'un callback apparaît dans l'UI

### 6.4 — Matrice de test des commandes

| Command | Input | Expected | Vérifié |
|---------|-------|----------|---------|
| whoami | - | user@host | ⬜ |
| pwd | - | chemin courant | ⬜ |
| ls | - | listing trié | ⬜ |
| ls /tmp | {"path":"/tmp"} | listing /tmp | ⬜ |
| cd /tmp | {"path":"/tmp"} | vide, pwd=/tmp | ⬜ |
| shell id | "id" | output de id | ⬜ |
| sleep 30 10 | {"seconds":30,"jitter":10} | confirmation | ⬜ |
| killdate 9999999999 | {"date":"9999999999"} | confirmation | ⬜ |
| exit | - | callback disparaît | ⬜ |

### 6.5 — Crypto cross-test Go ↔ Rust

Vérifier que `encryptCallback` (Go) est déchiffré par `decrypt_config` (Rust).
Implicitement vérifié par un checkin réussi.

---

## Phase 7 — Upload et Download via Mythic file store ⬜

### 7.1 — Download natif Mythic

L'implant doit utiliser le Mythic file transfer API :
1. `post_response` avec `file_browser` ou `upload`/`download` Mythic-specific keys
2. Chunking pour les gros fichiers
3. Registration du fichier dans le file store Mythic

### 7.2 — Upload natif Mythic

Recevoir le file UUID via les task parameters, récupérer le contenu via l'API Mythic,
écrire sur le disque cible.

### 7.3 — Adapter le dispatch Rust

Remplacer les stubs `"FILE:path:base64"` par le format Mythic.

---

## Phase 8 — Hardening et OPSEC ⬜

### 8.1 — `obfstr!()` sur les strings sensibles (D1)

Wrapper `"checkin"`, `"get_tasking"`, `"post_response"`, action strings avec `s!()`.

### 8.2 — Vrai secret AES (D2)

Générer un secret 32 bytes aléatoire séparé du PayloadUUID pour IMPLANT_SECRET.

### 8.3 — Sleep jitter sur le retry checkin (D3)

Ajouter backoff + jitter au retry loop du checkin.

### 8.4 — Zeroize les clés en mémoire (QUAL-01)

Utiliser `zeroize` pour effacer `encryption_key` en fin de `run_c2_loop`.

### 8.5 — Éliminer les panics dans le code crypto (QUAL-07)

Remplacer tous les `.expect()` dans le code crypto par des `Result` propagés.

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
| D1 | `obfstr!()` sur les strings d'action Mythic | Phase 8 |
| D2 | IMPLANT_SECRET = hex(UUID) est faible (16 bytes d'entropie sur 32) | Phase 8 |
| D3 | Pas de jitter sur le retry checkin | Phase 8 |
| D4 | macOS cross-compilation nécessite osxcross (non inclus dans le Dockerfile) | Deferred |
| D5 | URI hardcodé à `"/`" | ✅ Done (Phase 5) |
| D6 | Pre-commit hook Rust workspace validation | ✅ Done |
| D7 | reqwest version compatibility | ✅ Done — `"rustls"` correct en reqwest 0.13 |
| D8 | Test scripts are stubs | Phase 9 |

---

## File checklist — état actuel (phases 0-5c complètes)

```
agent_code/
└── links/
    ├── common/
    │   ├── Cargo.toml                 ✅ features = ["blocking", "json", "rustls"]
    │   └── src/
    │       ├── lib.rs                 ✅ build_client, run_c2_loop(callback_uri), error status
    │       └── dispatch.rs            ✅ dispatch_common avec clés JSON correctes par commande
    ├── linux/
    │   ├── build.rs                   ✅ CALLBACK_URI
    │   └── src/
    │       ├── main.rs                ✅ const CALLBACK_URI
    │       └── stdlib.rs              ✅ dispatch_common unifié
    ├── windows/
    │   ├── build.rs                   ✅ CALLBACK_URI
    │   └── src/
    │       ├── main.rs                ✅ const CALLBACK_URI
    │       └── stdlib.rs              ✅ dispatch_common unifié
    └── osx/
        ├── build.rs                   ✅ CALLBACK_URI
        └── src/
            ├── main.rs                ✅ const CALLBACK_URI
            └── stdlib.rs              ✅ dispatch_common unifié

(root)/
├── main.go                            ✅
├── go.mod                             ✅
├── Dockerfile                         ✅
└── mythic/
    ├── payload_type.go                ✅ callback_uri build param
    └── agent_functions/
        ├── builder.go                 ✅ PayloadUUID fix, debug path fix, CALLBACK_URI env
        ├── shell.go                   ✅
        ├── ls.go, cd.go, pwd.go ...   ✅
        ├── sleep.go                   ✅ structured params (seconds + jitter)
        ├── inject.go                  ✅ structured params (pid + shellcode)
        ├── exit.go                    ✅
        ├── download.go                ⬜ Phase 7 (Mythic file API)
        └── upload.go                  ⬜ Phase 7 (Mythic file API)
```