# ImageSorter

En desktop-applikasjon for å sortere bilder til mappestrukturer og finne duplikater ved hjelp av perceptuell hashing.

## Funksjoner

- 🖼️ **Bildesortering**: Organiser bilder i mappestrukturer basert på metadata, dato, eller manuell kategorisering
- 🔍 **Duplikatdeteksjon**: Finn duplikate og nesten-like bilder ved hjelp av:
  - Eksakt matching (fil-hash)
  - Perceptuell hashing (pHash, dHash, aHash)
  - Visuell likhetsammenligning
- ⚡ **Rask ytelse**: Rust-backend for effektiv bildebehandling
- 🎨 **Moderne UI**: Responsivt brukergrensesnitt bygget med webteknologi

## Teknologi

- **Frontend**: TypeScript, HTML, CSS (Vite)
- **Backend**: Rust (via Tauri v2)
- **Bildebehandling**: image-rs, img_hash

## Kom i gang

### Forutsetninger

- [Rust](https://rustup.rs/) (via rustup)
- [Node.js](https://nodejs.org/) 18+
- Linux: `sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libayatana-appindicator3-dev librsvg2-dev`

### Installasjon

```bash
git clone https://github.com/Hawk-on/ImageSorter.git
cd ImageSorter
npm install
npm run tauri dev
```

## Bidra til prosjektet

### Branching-strategi

Vi bruker en enkel branching-modell:

| Branch | Formål |
|--------|--------|
| `master` | Stabil, produksjonsklar kode |
| `dev` | Aktiv utvikling, neste release |
| `feature/*` | Nye funksjoner (brancher fra `dev`) |
| `fix/*` | Bugfikser (brancher fra `dev`) |

### Workflow

1. **Opprett feature branch** fra `dev`:
   ```bash
   git checkout dev
   git pull origin dev
   git checkout -b feature/min-nye-funksjon
   ```

2. **Gjør endringer** og commit:
   ```bash
   git add .
   git commit -m "feat: beskrivelse av endring"
   ```

3. **Push og lag Pull Request**:
   ```bash
   git push -u origin feature/min-nye-funksjon
   gh pr create --base dev --title "feat: beskrivelse"
   ```

4. **Etter godkjenning**: Merge til `dev`

5. **Release**: `dev` merges til `master` når features er testet

### Commit-konvensjoner

Vi følger [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: ny funksjonalitet
fix: bugfiks
docs: dokumentasjonsendringer
refactor: kodeomstrukturering
test: tester
chore: vedlikehold (deps, config)
```

## Prosjektstruktur

```
ImageSorter/
├── src/                    # Frontend (TypeScript)
│   ├── app.ts              # Hovedapplikasjon
│   ├── main.ts             # Entry point
│   └── styles/             # CSS
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── commands/       # Tauri IPC kommandoer
│   │   └── services/       # Forretningslogikk
│   └── Cargo.toml
├── docs/                   # Dokumentasjon
└── .agent/workflows/       # AI-assistent workflows
```

## Lisens

MIT
