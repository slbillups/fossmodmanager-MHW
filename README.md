# FossModManager

<p align="center">
  <img src="https://github.com/user-attachments/assets/6745e82e-d1c9-4e2b-a66d-af06d364b485)" alt="FossModManager Demo">
</p>

FossModManager is a local, offline mod manager for Linux, that is nowhere close to being finished. But you can still use it to manage your mods, toggle exisiting mods on and off, delete and install new mods. I may continue to work on this, but this was intended to just be a proof of concept.

**Version:** 0.7.0 alpha(?)

## Project Status

- This was originally intended to be a mod manager for multiple games, but has since been refocused on Monster Hunter Wilds. Writing a mod manager, especially one in Rust is not as fun as I thought it would be.
- This does have the functionality to install and track mods installed via .zip files(no RAR support yet because I couldn't get the unrar or xcompress crates to work nicely with zip/tauri), as well as tracking loose folders placed in the directory(which I currently have setup to watch for changes to skins).

Place your skins in the following directory:

```sh
$root_game_directory/fossmodmanager/mods
```

Most skins will include two subdirectories modname_tex modname_model, be sure to bring both up into the mods directory and enable both to enable the skin.


- The search feature is currently a **proof of concept**. You'll need to provide your own [Nexus Mods developer API key](https://www.nexusmods.com/users/myaccount?tab=api%20access) and place it in the root directory in an .env file with the key NEXUS_API_KEY=your_key_here.

- By default, opening the search window will only display the top 10 Monster Hunter Wilds mods.

- If things stop working, or mods aren't updating/the cache is invalidated - use the nuke button at the bottom of settings to clear the cache/config directories which should set you back to the setup page.


## Recommended IDE Setup

- Your IDE of choice + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

This project is licensed under GPLv3. 

### Fonts

This project uses the **Crimson Text** font, licensed under the [SIL Open Font License, Version 1.1](https://scripts.sil.org/OFL).
