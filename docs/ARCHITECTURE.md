# Heimdall Sort - Arkitekturdokumentasjon

## Oversikt

Heimdall Sort er en desktop-applikasjon bygget med Tauri v2 som kombinerer en TypeScript/HTML frontend med en Rust backend for effektiv bildebehandling.

## Arkitekturdiagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Desktop Application                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐    │
│  │                   Frontend (WebView)                  │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │    │
│  │  │  Components │ │   Services  │ │    State    │    │    │
│  │  │  - Gallery  │ │  - Tauri    │ │  - Images   │    │    │
│  │  │  - Sidebar  │ │    Bridge   │ │  - Folders  │    │    │
│  │  │  - Compare  │ │  - Events   │ │  - Settings │    │    │
│  │  └─────────────┘ └─────────────┘ └─────────────┘    │    │
│  └─────────────────────────────────────────────────────┘    │
│                            │ IPC                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                 Backend (Rust/Tauri)                  │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │    │
│  │  │  Commands   │ │  Services   │ │    Utils    │    │    │
│  │  │ - scan_dir  │ │ - Hashing   │ │  - Image    │    │    │
│  │  │ - find_dups │ │ - Scanner   │ │    decode   │    │    │
│  │  │ - move_file │ │ - Sorter    │ │  - Thumb    │    │    │
│  │  └─────────────┘ └─────────────┘ └─────────────┘    │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │   Filsystemet     │
                    │ - Bilder          │
                    │ - Mapper          │
                    │ - Cache           │
                    └───────────────────┘
```

## Hovedkomponenter

### Frontend

| Komponent | Ansvar |
|-----------|--------|
| **Gallery** | Viser bilder i grid/liste med thumbnails |
| **Sidebar** | Mappenavigasjon og filtrering |
| **Compare** | Side-by-side sammenligning av duplikater |
| **Settings** | Brukerkonfigurasjon |

### Backend (Rust)

| Service | Ansvar |
|---------|--------|
| **Scanner** | Traverserer mapper og finner bildefiler |
| **Hashing** | Beregner perceptuelle og fil-hasher |
| **Sorter** | Flytter/kopierer filer basert på dato/metadata |
| **Cache** | Persistent JSON-lagring av hasher |

## Duplikatdeteksjon

### Algoritmer

1. **Fil-hash (MD5/SHA-256)**
   - Rask første-pass
   - Finner kun eksakte duplikater

2. **Perceptuell Hashing**
   - **aHash (Average Hash)**: Enkel, rask, følsom for fargeendringer
   - **dHash (Difference Hash)**: Bedre for rotasjon/skalering
   - **pHash (Perceptual Hash)**: Mest robust, tregere

3. **Hamming Distance**
   - Sammenligner perceptuelle hasher
   - Terskelverdi bestemmer "likhet"

### Arbeidsflyt

```
SKann mappe → Beregn hasher (sjekk JSON cache) → Bygg BK-Tree → Søk naboer → Grupper
     │                   │                             │               │
     ▼                   ▼                             ▼               ▼
  Parallell    Cache hits sparer CPU             O(N log N)       O(N) effektiv
  traversering                                   søk               gruppering
```

## Datamodeller

### Rust Types

```rust
struct ImageFile {
    path: PathBuf,
    file_hash: Option<String>,
    perceptual_hash: Option<Vec<u8>>,
    metadata: ImageMetadata,
}

struct ImageMetadata {
    size_bytes: u64,
    dimensions: (u32, u32),
    format: ImageFormat,
    created_at: Option<DateTime<Utc>>,
    exif: Option<ExifData>,
}

struct DuplicateGroup {
    primary: ImageFile,
    duplicates: Vec<ImageFile>,
    similarity: f32,
}
```

### Frontend Types

```typescript
interface ImageFile {
    path: string;
    thumbnail: string;
    metadata: ImageMetadata;
}

interface DuplicateGroup {
    primary: ImageFile;
    duplicates: ImageFile[];
    similarity: number;
}
```

### 3. Services (`src-tauri/src/services/`)
- **scanner.rs**: Rekursiv filskanning (WalkDir), optimalisert for ytelse.
- **hashing.rs**: Bildehashing (pHash, BK-Tree) for duplikatdeteksjon.
- **thumbnail.rs**: Generering og caching av thumbnails for rask visning.
- **metadata.rs**: Leser EXIF-data for sortering.
- **sorter.rs**: Håndterer filoperasjoner (sortering, sletting, flytting).
- **cache.rs**: Persistent lagring av hasher for å unngå reskanning.

### 4. Viktige Biblioteker
- `tauri`: Rammeverk.
- `rayon`: Parallell prosessering.
- `img_hash`: Perceptuell hashing.
- `bk-tree`: Effektivt søk etter lignende bilder (O(N log N)).
- `trash`: Sikker sletting til papirkurv.
- `kamadak-exif`: Metadata-lesing.

## Ytelsesoptimalisering

1. **Parallell prosessering**: Bruk Rayon for CPU-bundet arbeid
2. **Thumbnail caching**: Forhåndsgenererte thumbnails
3. **Hash caching**: Lagre beregnede hasher i JSON-fil (`hash_cache.json`)
4. **Virtuell Scrolling**: "Windowing" - rendrer kun synlige elementer (håndterer 10k+ bilder)
5. **Debouncing**: Optimalisert UI-respons ved resizing og input
```
