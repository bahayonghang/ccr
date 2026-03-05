# MapleBright Web Fonts

This directory stores the subsetted `woff2` assets used by CCR-UI.

## Source

- Repository: `https://github.com/bahayonghang/MapleLXGWBright.git`
- Pinned commit: `c0f2af4ffad1a1d68b6e873ee416e4cf2a8ce05d`
- Family: `MapleBright`
- Variants:
  - `MapleBright-Regular` (`400`, normal)
  - `MapleBright-Medium` (`500`, normal)
  - `MapleBright-Italic` (`400`, italic)
  - `MapleBright-MediumItalic` (`500`, italic)

## Regeneration

Run the script below from repo root:

```powershell
pwsh -File "./ccr-ui/scripts/build-maplebright-fonts.ps1"
```

The script rebuilds and re-splits fonts, then writes assets back to this folder.
