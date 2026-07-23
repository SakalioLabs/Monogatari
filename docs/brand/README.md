# Monogatari brand assets

The Monogatari mark combines three project ideas in one compact symbol:

- an open book for authored narrative;
- an abstract **M** formed by the pages and spine;
- a vermilion branching node for bounded, AI-assisted story paths.

## Palette

| Role | Color |
|---|---|
| Ink | `#17181C` |
| Paper | `#F4F0E8` |
| Paper shadow | `#E7E0D4` |
| Narrative node | `#E4573D` |
| Secondary text | `#5F6065` |

## Files

- `logo-mark.svg` — scalable standalone mark.
- `logo-lockup.svg` — horizontal project wordmark for light surfaces.
- `logo-lockup.png` — raster fallback for platforms that cannot render SVG.
- `logo-mark.png` — 1024 px raster mark for social and marketplace use.

The PWA SVG icons under `frontend/public/` and Windows icons under
`rust-engine/crates/tauri-app/icons/` are derived from the same mark.

Keep the mark’s proportions and colors intact. Use the maskable icon where an
operating system may crop the image into a circle or rounded shape.
