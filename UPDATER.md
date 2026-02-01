# Strategie de Mise à Jour (Tauri Auto-Updater)

> [!NOTE]
> Cette tâche est reportée jusqu'à la sortie d'une version de base stable.

## 🚀 Étapes d'implémentation

### 1. Génération des Clés (Indispensable)
L'application doit être signée pour que Tauri accepte de la remplacer.
1. Générer les clés : `pnpm tauri signer generate -w src-tauri/id_tauri`
2. **id_tauri** (clé privée) : À stocker précieusement (ex: GitHub Secret `TAURI_PRIVATE_KEY`).
3. **id_tauri.pub** (clé publique) : À copier dans `tauri.conf.json`.

### 2. Configuration du Plugin
- Installer `tauri-plugin-updater` et `tauri-plugin-dialog`.
- Ajouter la configuration dans `tauri.conf.json` :
```json
{
  "plugins": {
    "updater": {
      "endpoints": ["https://raw.githubusercontent.com/username/RouxFlow/main/update.json"],
      "pubkey": "VOTRE_CLE_PUBLIQUE_ICI"
    }
  }
}
```

### 3. Workflow CI/CD (GitHub Actions)
Le workflow doit :
1. Compiler le binaire.
2. Signer le binaire avec la clé privée.
3. Générer le fichier `update.json` pointant vers la nouvelle release.

### 4. Interface Utilisateur (App.vue)
Exemple de logique de vérification :
```typescript
import { checkUpdate, installUpdate } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

const update = await checkUpdate()
if (update?.available) {
  // Afficher un bandeau ou une modal
  await installUpdate()
  await relaunch()
}
```
