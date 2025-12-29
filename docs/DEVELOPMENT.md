# Utviklingsguide

## Git Workflow

### Branching

```
master (stabil)
  └── dev (aktiv utvikling)
       ├── feature/gallery-view
       ├── feature/duplicate-detection
       └── fix/dialog-permission
```

### Daglig utvikling

```bash
# Start på dev
git checkout dev
git pull origin dev

# Lag feature branch
git checkout -b feature/beskrivende-navn

# Jobb, commit ofte
git add .
git commit -m "feat: kort beskrivelse"

# Push og lag PR
git push -u origin feature/beskrivende-navn
gh pr create --base dev
```

### Pull Requests

- PR skal alltid gå til `dev`, ikke `master`
- Beskriv hva endringen gjør
### Installasjon (Første gang)

```bash
git clone https://github.com/Hawk-on/HeimdallSort.git
cd HeimdallSort
npm install

# Last ned nødvendige binærfiler (ffmpeg/ffprobe) for videostøtte
npm run setup 

npm run tauri dev
```

### Pull Requests

- PR skal alltid gå til `dev`, ikke `master`
- Beskriv hva endringen gjør
- Legg til screenshots for UI-endringer
- Sørg for at koden bygger (`npm run tauri build`)
- Sørg for at dependencies er lastet ned (`npm run setup`)

## Kodekonvensjoner

### TypeScript

- Async/await fremfor callbacks
- Streng typing (unngå `any`)
- ESLint + Prettier

```typescript
// ✅ God praksis
async function loadImages(directory: string): Promise<ImageInfo[]> {
    return await invoke<ImageInfo[]>('scan_folder', { path: directory });
}

// ❌ Unngå
function loadImages(directory, callback) {
    invoke('scan_folder', { path: directory }).then(callback);
}
```

### Rust

- Følg clippy-anbefalinger
- `Result<T, E>` for feilhåndtering
- Doc comments på public API

```rust
/// Skanner en mappe rekursivt etter bilder.
///
/// # Arguments
/// * `path` - Sti til mappen som skal skannes
///
/// # Returns
/// Liste med bildeinformasjon, eller feil
pub fn scan_directory(path: &str) -> Result<Vec<ImageInfo>, ScanError> {
    // ...
}
```

## Filnavnkonvensjoner

| Type | Konvensjon | Eksempel |
|------|------------|----------|
| TypeScript | camelCase | `imageService.ts` |
| Rust | snake_case | `image_service.rs` |
| CSS (Modul) | kebab-case | `src/styles/modules/gallery.css` |
| CSS (Komp) | kebab-case | `src/styles/components/buttons.css` |

## Commit-meldinger

```
feat: legg til bildegalleri
fix: rett tilgangsfeil i dialog
docs: oppdater README med git-workflow
refactor: omstrukturer scanner-modul
test: legg til enhetstester for hashing
chore: oppdater dependencies
```

## Testing

### Frontend
```bash
npm test              # Kjør tester
npm run test:watch    # Watch mode
```

### Backend (Rust)
```bash
cd src-tauri
cargo test            # Kjør tester
cargo clippy          # Lint-sjekk
```

## Debugging

### Frontend
- DevTools: Høyreklikk → Inspect (eller F12)
- Tauri console: Se Rust-output i terminalen

### Backend
```bash
RUST_LOG=debug npm run tauri dev   # Verbose logging
```

## Vanlige oppgaver

### Legge til ny Tauri-kommando

1. Definer i `src-tauri/src/commands/`:
   ```rust
   #[tauri::command]
   pub async fn min_kommando(arg: String) -> Result<String, String> {
       Ok(format!("Mottok: {}", arg))
   }
   ```

2. Registrer i `main.rs`:
   ```rust
   .invoke_handler(tauri::generate_handler![
       commands::folder::scan_folder,
       commands::folder::min_kommando
   ])
   ```

3. Kall fra frontend:
   ```typescript
   const result = await invoke<string>('min_kommando', { arg: 'test' });
   ```

### Legge til ny permission

Oppdater `src-tauri/capabilities/main.json`:
```json
{
    "permissions": [
        "core:default",
        "dialog:allow-open",
        "fs:read-all",
        "ny-permission:her"
    ]
}
```
