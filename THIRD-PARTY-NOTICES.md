# Third-party notices

MANTIS POSTURE is licensed under MIT. That license covers the project’s own
code and documentation only. Dependencies, bundled tools, trademarks and
third-party assets keep their respective rights.

## Bundled Windows sidecars

| Component | Pinned version | License and source |
| --- | --- | --- |
| DDGS | 9.14.4 | [upstream](https://github.com/deedy5/ddgs) |
| User Scanner | 1.4.2.1 | MIT · [upstream](https://github.com/kaifcodec/user-scanner) |
| Maigret | 0.6.3 | [upstream](https://github.com/soxoj/maigret) |
| PyInstaller | 6.21.0 | GPL-2.0-or-later with distribution exception · [license](https://github.com/pyinstaller/pyinstaller/blob/main/COPYING.txt) |

The build scripts copy the upstream license supplied by each packaged project
into its resource directory. Those notices must remain in the Windows bundle.

## Main application dependencies

| Component | License |
| --- | --- |
| Tauri and official plugins | MIT OR Apache-2.0 |
| Svelte and SvelteKit | MIT |
| Cytoscape.js | MIT |
| Vite | MIT |
| TypeScript | Apache-2.0 |
| Rust crates | See `Cargo.lock` and each crate’s package metadata |
| JavaScript packages | See `package-lock.json` and each package’s metadata |

This list highlights direct components; it is not a complete transitive
license report. A dependency-license scan remains required for every release
artifact.

## Visual assets and trademarks

Names such as Signal, Proton, GrapheneOS, OpenAI and Facebook, and their logos,
are trademarks of their respective owners. Their appearance does not imply
endorsement or affiliation.

The MIT license must not be interpreted as relicensing third-party screenshots.
