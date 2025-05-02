# FossModManager

FossModManager is a mod manager for Monster Hunter Wilds, built with Tauri and React.  
This project is still in active development, and isn't intended for public use yet.  However you're free to download, fork, and modify the code as you see fit.

**Version:** 0.7.0 (testing checkpoint)

## Project Status

- This was originally intended to be a mod manager for multiple games, but has since been refocused on Monster Hunter Wilds. Writing a mod manager, especially one in Rust is a bit more of a challenge than I anticipated.
- This does have the functionality to install and track mods installed via .zip files, as well as tracking loose folders placed in the directory(which I currently have setup to watch for changes to skins):

```sh
$root_game_directory/fossmodmanager/mods
```

- The search feature is currently a **proof of concept**. You'll need to provide your own [Nexus Mods developer API key](https://www.nexusmods.com/users/myaccount?tab=api%20access) and modify the configuration accordingly if you want to use/modify it.
- By default, opening the search window will only display the top 10 Monster Hunter Wilds mods.


## Usage

Anyone is free to download and use this application as-is, but please be aware of the limitations above.

## Recommended IDE Setup

- Your IDE of choice + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

This project is licensed under the MIT License.

### Fonts

This project uses the **Crimson Text** font, licensed under the [SIL Open Font License, Version 1.1](https://scripts.sil.org/OFL).
