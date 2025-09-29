# cave — code_aster Version Manager (Developer Guide)

`cave` est un outil en ligne de commande écrit en **Rust** qui permet de gérer plusieurs versions de **code_aster** via Docker.  
Ce dépôt contient le code source et l’infrastructure nécessaire au développement et à la maintenance du CLI.

---

## Prérequis

- **Rust** (version stable, installée via [rustup](https://rustup.rs/))  
- **Cargo** (fourni avec Rust)  
- **Docker** 
- **proto Compiler**, à installer avec : 

```bash 
sudo apt-get update
sudo apt-get install protobuf-compiler
```

---

## Cloner et builder

```bash
git clone git@gitlab.com:simvia/common-tools/cave.git
cd cave
cargo build
```

Pour builder en mode release :
```bash
cargo build --release
```

---

## Documentation

La documentation se télécharge en local (doc html) en executant :
```bash
cargo doc
```
, vous pouvez alors visualiser la doc via ce chemin : target/doc/cave/index.html. 

---

## Executer le CLI 

Vous pouvez essayer vos implémentations avec : 
```bash
cargo run -- [args]
```
, où [args] sont les arguments classiques de cave (cette commande est équivalente à cave [args], une fois le binaire produit).

---

## Tests

Lancer les tests scénarios :
```bash
cargo test
```

---

## Variables d'environnement

Le CLI supporte deux variables d'environnement optionnelles :

1. `CAVE_DEBUG=true`  
   - Active l'affichage des traces en mode debug du CLI. Utile pour suivre les opérations internes et déboguer des problèmes.

2. `LOCAL_TELEMETRY=true`  
   - Force l'envoi de la télémétrie vers un serveur local sur un port personnalisé, au lieu du serveur distant par défaut Simvia.  
   - Utile pour tester la télémétrie en local sans impacter la production.

Pour les utiliser, exportez simplement les variables avant de lancer le CLI, par exemple :

```bash
export CAVE_DEBUG=true
export LOCAL_TELEMETRY=true
cave your-command
```

## Structure du projet

- `src/main.rs` → point d’entrée CLI  
- `src/cli.rs` → définition des sous-commandes (via `clap`)  
- `src/config.rs` → gestion de la configuration utilisateur globale (`~/.caveconfig.json`)  
- `src/manage.rs` → gestion des versions et des erreurs personnalisées   

---

## Ajouter une nouvelle commande CLI

1. Ajouter une **sous-commande** dans `cli.rs` (via `ConfigAction` ou `CaveAction`).  
2. Implémenter son **handler** dans `main.rs`.  
3. Si nécessaire, créer un module dédié dans `src/`.  
4. Mettre à jour la **page man**  la documentation.  
5. Ajouter des **tests scenarios**.

---

## Gestion de la configuration

La configuration globale est stockée dans `~/.caveconfig.json`.  

Pour ajouter une nouvelle option :
1. Ajouter un champ dans `Config` (dans `config.rs`).  
2. Étendre `Default::default`.  
3. Créer un setter (comme `set_auto_update`).  
4. Étendre la CLI (`cli.rs`) et `main.rs`.  

---

## Distribution

- Le binaire est copié dans `~/.cave_cmd/bin` puis ajouté au `PATH`.  
- L’autocomplétion peut être activée via :
  ```bash
  source /chemin/vers/_cave >> ~/.zshrc
  ```

---

## Bonnes pratiques de développement

- Documenter toutes les **fonctions publiques** avec Rustdoc (exemple + usage).  
- Garder le code idiomatique (respecter `cargo fmt` et `cargo clippy`).  
- Isoler les erreurs dans `CaveError`.  
- Tester chaque nouvelle fonctionnalité avec `cargo test`.  

---
